mod sequencer;
mod oscillator;
mod length_counter;
mod envelope;
mod sweeper;

use sequencer::Sequencer;
use length_counter::LengthCounter;
use envelope::Envelope;
use sweeper::Sweeper;
use oscillator::OscillatorPulse;

use crate::utils::consts::{LENGTH_TABLE, NTSC_CRYSTAL_FREQUENCY};

pub struct APU {
    pub global_time: f64,
    frame_count_sequence: bool,
    frame_count_interrupt: bool,
    pub audio_sample_rate: f32,

    pulse1_enable: bool,
    pulse1_halt: bool,
    pulse1_sample: f32,
    pulse1_output: f32,
    pulse1_seq: Sequencer,
    pulse1_osc: OscillatorPulse,
    pulse1_env: Envelope,
    pulse1_lc: LengthCounter,
    pulse1_sweep: Sweeper,

    pulse2_enable: bool,
    pulse2_halt: bool,
    pulse2_sample: f32,
    pulse2_output: f32,
    pulse2_seq: Sequencer,
    pulse2_osc: OscillatorPulse,
    pulse2_env: Envelope,
    pulse2_lc: LengthCounter,
    pulse2_sweep: Sweeper,

    noise_enable: bool,
    noise_halt: bool,
    noise_sample: f32,
    noise_output: f32,
    noise_env: Envelope,
    noise_lc: LengthCounter,
    noise_seq: Sequencer,
    noise_mode: bool,
    

    frame_clock_counter: u32,
    clock_counter: u32,
    use_raw_mode: bool,
    last_reload: u16,
    
}

impl APU {
    pub fn new() -> APU {
        let mut apu= APU {
            global_time: 0.0,
            frame_count_sequence: false,
            frame_count_interrupt: false,
            audio_sample_rate: 44100.0,

            pulse1_enable: false,
            pulse1_halt: false,
            pulse1_sample: 0.0,
            pulse1_output: 0.0,
            pulse1_seq: Sequencer::new(),
            pulse1_osc: OscillatorPulse::new(),
            pulse1_env: Envelope::new(),
            pulse1_lc: LengthCounter::new(),
            pulse1_sweep: Sweeper::new(),

            pulse2_enable: false,
            pulse2_halt: false,
            pulse2_sample: 0.0,
            pulse2_output: 0.0,
            pulse2_seq: Sequencer::new(),
            pulse2_osc: OscillatorPulse::new(),
            pulse2_env: Envelope::new(),
            pulse2_lc: LengthCounter::new(),
            pulse2_sweep: Sweeper::new(),
            
            noise_enable: false,
            noise_halt: false,
            noise_sample: 0.0,
            noise_output: 0.0,
            noise_env: Envelope::new(),
            noise_lc: LengthCounter::new(),
            noise_seq: Sequencer::new(),
            noise_mode: true,
            

            frame_clock_counter: 0,
            use_raw_mode: false,
            clock_counter: 0,
            last_reload: 0,   
        };

        apu.noise_seq.sequence = 0x004;

        apu
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            0x4000 => {
                match (data & 0xC0) >> 6 {
                    // 0 => {self.pulse1_seq.new_sequence = 0b01000000; self.pulse1_osc.duty_cycle = 0.125},
                    // 1 => {self.pulse1_seq.new_sequence = 0b01100000; self.pulse1_osc.duty_cycle = 0.250},
                    // 2 => {self.pulse1_seq.new_sequence = 0b01111000; self.pulse1_osc.duty_cycle = 0.500},
                    // 3 => {self.pulse1_seq.new_sequence = 0b10011111; self.pulse1_osc.duty_cycle = 0.750},
                    0 => {self.pulse1_seq.new_sequence = 0b00000001; self.pulse1_osc.duty_cycle = 0.125},
                    1 => {self.pulse1_seq.new_sequence = 0b00000011; self.pulse1_osc.duty_cycle = 0.250},
                    2 => {self.pulse1_seq.new_sequence = 0b00001111; self.pulse1_osc.duty_cycle = 0.500},
                    3 => {self.pulse1_seq.new_sequence = 0b11111100; self.pulse1_osc.duty_cycle = 0.750},
                    _ => ()
                };
                self.pulse1_seq.sequence = self.pulse1_seq.new_sequence;
                self.pulse1_halt = (data & 0x20) != 0;
                self.pulse1_env.volume = (data & 0x0F) as u16;
                self.pulse1_env.disable = (data & 0x10) != 0;
            },
            0x4001 => {
                self.pulse1_sweep.enabled = (data & 0x80) != 0;
                self.pulse1_sweep.period = ((data & 0x70) >> 4) + 1;
                self.pulse1_sweep.down = (data & 0x08) != 0;
                self.pulse1_sweep.shift = data & 0x07;
                self.pulse1_sweep.reload = true;
            },
            0x4002 => {
                self.pulse1_seq.reload = (self.pulse1_seq.reload & 0xFF00) | data as u16;
            },
            0x4003 => {
                self.pulse1_seq.reload = ((((data & 0x07) as u16) << 8) | (self.pulse1_seq.reload & 0x00FF)) - 1;
                self.pulse1_seq.timer = self.pulse1_seq.reload;
                self.pulse1_seq.sequence = self.pulse1_seq.new_sequence;
                self.pulse1_lc.counter = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
                self.pulse1_env.start = true;
            },
            0x4004 => {
                match (data & 0xC0) >> 6 {
                    // 0 => {self.pulse2_seq.new_sequence = 0b01000000; self.pulse2_osc.duty_cycle = 0.125},
                    // 1 => {self.pulse2_seq.new_sequence = 0b01100000; self.pulse2_osc.duty_cycle = 0.250},
                    // 2 => {self.pulse2_seq.new_sequence = 0b01111000; self.pulse2_osc.duty_cycle = 0.500},
                    // 3 => {self.pulse2_seq.new_sequence = 0b10011111; self.pulse2_osc.duty_cycle = 0.750},
                    0 => {self.pulse2_seq.new_sequence = 0b00000001; self.pulse2_osc.duty_cycle = 0.125},
                    1 => {self.pulse2_seq.new_sequence = 0b00000011; self.pulse2_osc.duty_cycle = 0.250},
                    2 => {self.pulse2_seq.new_sequence = 0b00001111; self.pulse2_osc.duty_cycle = 0.500},
                    3 => {self.pulse2_seq.new_sequence = 0b11111100; self.pulse2_osc.duty_cycle = 0.750},
                    _ => ()
                };
                self.pulse2_seq.sequence = self.pulse2_seq.new_sequence;
                self.pulse2_halt = (data & 0x20) != 0;
                self.pulse2_env.volume = (data & 0x0F) as u16;
                self.pulse2_env.disable = (data & 0x10) != 0;
            },
            0x4005 => {
                self.pulse2_sweep.enabled = (data & 0x80) != 0;
                self.pulse2_sweep.period = ((data & 0x70) >> 4) + 1;
                self.pulse2_sweep.down = (data & 0x08) != 0;
                self.pulse2_sweep.shift = data & 0x07;
                self.pulse2_sweep.reload = true;
            },
            0x4006 => {
                self.pulse2_seq.reload = (self.pulse2_seq.reload & 0xFF00) | data as u16;
            },
            0x4007 => {
                self.pulse2_seq.reload = (((data as u16 & 0x07) << 8) | (self.pulse2_seq.reload & 0x00FF)) - 1;
                self.pulse2_seq.timer = self.pulse2_seq.reload;
                self.pulse2_seq.sequence = self.pulse2_seq.new_sequence;
                self.pulse2_lc.counter = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
                self.pulse2_env.start = true;
            },
            0x400C => {
                self.noise_env.volume = (data & 0x0F) as u16;
                self.noise_env.disable = (data & 0x10) != 0;
                self.noise_halt = (data & 0x20) != 0;
            },
            0x400E => {
                match data & 0x0F {
                    0 => self.noise_seq.reload = 0x004,
                    1 => self.noise_seq.reload = 0x008,
                    2 => self.noise_seq.reload = 0x010,
                    3 => self.noise_seq.reload = 0x020,
                    4 => self.noise_seq.reload = 0x040,
                    5 => self.noise_seq.reload = 0x060,
                    6 => self.noise_seq.reload = 0x080,
                    7 => self.noise_seq.reload = 0x0A0,
                    8 => self.noise_seq.reload = 0x0CA,
                    9 => self.noise_seq.reload = 0x0FE,
                    10 => self.noise_seq.reload = 0x17C,
                    11 => self.noise_seq.reload = 0x1FC,
                    12 => self.noise_seq.reload = 0x2FA,
                    13 => self.noise_seq.reload = 0x3F8,
                    14 => self.noise_seq.reload = 0x7F2,
                    15 => self.noise_seq.reload = 0xFE4,
                    _ => ()
                }
                self.noise_mode = (data & 0x80) >> 7 != 0;
            },
            0x4015 => {
                self.pulse1_enable = (data & 0x01) != 0;
                self.pulse2_enable = (data & 0x02) != 0;
                self.noise_enable = (data & 0x04) != 0;
            },
            0x4017 => {
                self.frame_count_interrupt = (data & 0x40) == 0;
                self.frame_count_sequence = (data & 0x80) == 0;
            },
            0x400F => {
                self.pulse1_env.start = true;
                self.pulse2_env.start = true;
                self.noise_env.start = true;
                self.noise_lc.counter = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
            }
            _ => ()
        }
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        0
    }

    pub fn clock(&mut self) {
        let mut quarter_frame_clock = false;
        let mut half_frame_clock = false;

        self.global_time += 1.0 / NTSC_CRYSTAL_FREQUENCY as f64;

        if self.clock_counter % 6 == 0 {
            self.frame_clock_counter += 1;

            if self.frame_count_sequence {
                match self.frame_clock_counter {
                    3729 => {
                        quarter_frame_clock = true;
                    }
                    
                    7457 => {
                        quarter_frame_clock = true;
                        half_frame_clock = true;
                    }
                    
                    11186 => {
                        quarter_frame_clock = true;
                    }
                    
                    14915 => {
                        quarter_frame_clock = true;
                        half_frame_clock = true;
                        self.frame_clock_counter = 0;
                    }

                    _ => ()
                }
            } else {
                match self.frame_clock_counter {
                    3729 => {
                        quarter_frame_clock = true;
                    }
                    
                    7457 => {
                        quarter_frame_clock = true;
                        half_frame_clock = true;
                    }
                    
                    11186 => {
                        quarter_frame_clock = true;
                    }
                    
                    14915 => {
                        
                    }

                    18641 => {
                        quarter_frame_clock = true;
                        half_frame_clock = true;
                        self.frame_clock_counter = 0;
                    }

                    _ => ()
                }
            }

            if quarter_frame_clock {
                self.pulse1_env.clock(self.pulse1_halt);
                self.pulse2_env.clock(self.pulse2_halt);
                self.noise_env.clock(self.noise_halt);
            }

            if half_frame_clock {
                self.pulse1_lc.clock(self.pulse1_enable, self.pulse1_halt);
                self.pulse2_lc.clock(self.pulse2_enable, self.pulse2_halt);
                self.noise_lc.clock(self.noise_enable, self.noise_halt);
                self.pulse1_sweep.clock(&mut self.pulse1_seq.reload, false);
                self.pulse2_sweep.clock(&mut self.pulse2_seq.reload, true);
            }

            self.pulse1_seq.clock(self.pulse1_enable, |s| {
                *s = ((*s & 0x0001) << 7) | ((*s & 0x00FE) >> 1);
            });

            self.pulse1_osc.frequency = (NTSC_CRYSTAL_FREQUENCY / 3.0) / (16.0 * (self.pulse1_seq.reload as f32 + 1.0));
            self.pulse1_osc.amplitude = self.pulse1_env.output as f32 - 1.0;
            self.pulse1_sample = self.pulse1_osc.sample(self.global_time as f32);

            if self.pulse1_lc.counter > 0 && self.pulse1_seq.timer >= 8 && !self.pulse1_sweep.mute && self.pulse1_env.output > 2 {
                self.pulse1_output += (self.pulse1_sample - self.pulse1_output) * 0.5;
            } else {
                self.pulse1_output = 0.0;
            }

            self.pulse2_seq.clock(self.pulse2_enable, |s| {
                *s = ((*s & 0x0001) << 7) | ((*s & 0x00FE) >> 1);
            });

            self.pulse2_osc.frequency = (NTSC_CRYSTAL_FREQUENCY / 3.0) / (16.0 * (self.pulse2_seq.reload as f32 + 1.0));
            self.pulse2_osc.amplitude = self.pulse2_env.output as f32 - 1.0;
            self.pulse2_sample = self.pulse2_osc.sample(self.global_time as f32);

            if self.pulse2_lc.counter > 0 && self.pulse2_seq.timer >= 8 && !self.pulse2_sweep.mute && self.pulse2_env.output > 2 {
                self.pulse2_output += (self.pulse2_sample - self.pulse2_output) * 0.5;
            } else {
                self.pulse2_output = 0.0;
            }

            self.noise_seq.clock(self.noise_enable, |s| {
                if !self.noise_mode {
                    *s = (((*s & 0x0001) ^ ((*s & 0x0002) >> 1)) << 14) | ((*s & 0x7FFF) >> 1);
                } else {
                    *s = (((*s & 0x0001) ^ ((*s & 0x0040) >> 6)) << 14) | ((*s & 0x7FFF) >> 1);
                }
            });

            if self.noise_lc.counter > 0 && self.noise_seq.timer >= 8 {
                self.noise_output = self.noise_seq.output as f32 * (self.noise_output - 1.0) as f32 / 1.15;
                // self.noise_output = self.noise_output * (self.noise_env.output as f32);
            }

            if !self.pulse1_enable {self.pulse1_output = 0.0};
            if !self.pulse2_enable {self.pulse2_output = 0.0};
            if !self.noise_enable {self.noise_output = 0.0};
        }

        self.pulse1_sweep.track(self.pulse1_seq.reload);
        self.pulse2_sweep.track(self.pulse2_seq.reload);
        self.clock_counter += 1;
    }

    pub fn reset(&mut self) {
    }

    pub fn get_output_sample(&mut self) -> f32 {
        let pulse_out = 95.88 / ((8128.0 / (self.pulse1_output + self.pulse2_output)) + 100.0);
        let tnd_out = 159.79 / ((1.0 / ((self.noise_output / 12241.0))) + 100.0);
        // println!("Reload: {:X}, Frequency: {}", self.pulse1_seq.reload, self.pulse1_osc.frequency);
        // self.pulse1_output + self.pulse2_output
        // (self.pulse1_output - 0.5) * 0.5 + (self.pulse2_output - 0.5) * 0.5 + (self.noise_output - 0.5) * 0.5
        pulse_out + tnd_out

        // ((1.0 * self.pulse1_output) - 0.8) * 0.1 +
		// ((1.0 * self.pulse2_output) - 0.8) * 0.1 +
		// ((1.0 * (self.noise_output - 0.8))) * 0.1
    }
}