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

use crate::utils::consts::LENGTH_TABLE;

pub struct APU {
    global_time: f32,

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
    

    frame_clock_counter: u32,
    clock_counter: u32,
    use_raw_mode: bool,
    
}

impl APU {
    pub fn new() -> APU {
        let mut apu= APU {
            global_time: 0.0,

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
            

            frame_clock_counter: 0,
            use_raw_mode: false,
            clock_counter: 0,   
        };

        apu.noise_seq.sequence = 0xDBDB;

        apu
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        //println!("APU write: {:04X} {:02X}", addr, data);
        match addr {
            0x4000 => {
                //println!("{}", (data & 0xC0));
                match (data & 0xC0) >> 6 {
                    0 => {self.pulse1_seq.new_sequence = 0b01000000; self.pulse1_osc.duty_cycle = 0.125},
                    1 => {self.pulse1_seq.new_sequence = 0b01100000; self.pulse1_osc.duty_cycle = 0.250},
                    2 => {self.pulse1_seq.new_sequence = 0b01111000; self.pulse1_osc.duty_cycle = 0.500},
                    3 => {self.pulse1_seq.new_sequence = 0b10011111; self.pulse1_osc.duty_cycle = 0.750},
                    _ => ()
                };
                self.pulse1_seq.sequence = self.pulse1_seq.new_sequence;
                self.pulse1_halt = (data & 0x20) != 0;
                self.pulse1_env.volume = (data & 0x0F) as u16;
                self.pulse1_env.disable = (data & 0x10) != 0;
                //println!("Pulse1 sequence: {:08b}", self.pulse1_seq.sequence);
            },
            0x4001 => {
                self.pulse1_sweep.enabled = (data & 0x80) != 0;
                self.pulse1_sweep.period = (data & 0x70) >> 4;
                self.pulse1_sweep.down = (data & 0x08) != 0;
                self.pulse1_sweep.shift = data & 0x07;
                self.pulse1_sweep.reload = true;
                //println!("Pulse1 sweep: {} {} {} {}", self.pulse1_sweep.enable, self.pulse1_sweep.period, self.pulse1_sweep.negate, self.pulse1_sweep.shift);
            },
            0x4002 => {
                self.pulse1_seq.reload = (self.pulse1_seq.reload & 0xFF00) | data as u16;
                //println!("Pulse1 reload: {:04X}", self.pulse1_seq.reload);
            },
            0x4003 => {
                self.pulse1_seq.reload = ((data as u16 & 0x07) << 8) | (self.pulse1_seq.reload & 0x00FF);
                self.pulse1_seq.timer = self.pulse1_seq.reload;
                self.pulse1_seq.sequence = self.pulse1_seq.new_sequence;
                self.pulse1_lc.counter = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
                self.pulse1_env.start = true;
                //println!("Pulse1 timer: {:04X}", self.pulse1_seq.timer);
            },
            0x4004 => {
                //println!("{}", (data & 0xC0));
                match (data & 0xC0) >> 6 {
                    0 => {self.pulse2_seq.new_sequence = 0b01000000; self.pulse2_osc.duty_cycle = 0.125},
                    1 => {self.pulse2_seq.new_sequence = 0b01100000; self.pulse2_osc.duty_cycle = 0.250},
                    2 => {self.pulse2_seq.new_sequence = 0b01111000; self.pulse2_osc.duty_cycle = 0.500},
                    3 => {self.pulse2_seq.new_sequence = 0b10011111; self.pulse2_osc.duty_cycle = 0.750},
                    _ => ()
                };
                self.pulse2_seq.sequence = self.pulse2_seq.new_sequence;
                self.pulse2_halt = (data & 0x20) != 0;
                self.pulse2_env.volume = (data & 0x0F) as u16;
                self.pulse2_env.disable = (data & 0x10) != 0;
                //println!("pulse2 sequence: {:08b}", self.pulse2_seq.sequence);
            },
            0x4005 => {
                self.pulse2_sweep.enabled = (data & 0x80) != 0;
                self.pulse2_sweep.period = (data & 0x70) >> 4;
                self.pulse2_sweep.down = (data & 0x08) != 0;
                self.pulse2_sweep.shift = data & 0x07;
                self.pulse2_sweep.reload = true;
                //println!("pulse2 sweep: {} {} {} {}", self.pulse2_sweep.enable, self.pulse2_sweep.period, self.pulse2_sweep.negate, self.pulse2_sweep.shift);
            },
            0x4006 => {
                self.pulse2_seq.reload = (self.pulse2_seq.reload & 0xFF00) | data as u16;
                //println!("pulse2 reload: {:04X}", self.pulse2_seq.reload);
            },
            0x4007 => {
                self.pulse2_seq.reload = ((data as u16 & 0x07) << 8) | (self.pulse2_seq.reload & 0x00FF);
                self.pulse2_seq.timer = self.pulse2_seq.reload;
                self.pulse2_seq.sequence = self.pulse2_seq.new_sequence;
                self.pulse2_lc.counter = LENGTH_TABLE[((data & 0xF8) >> 3) as usize];
                self.pulse2_env.start = true;
                //println!("pulse2 timer: {:04X}", self.pulse2_seq.timer);
            },
            0x400C => {
                self.noise_env.volume = (data & 0x0F) as u16;
                self.noise_env.disable = (data & 0x10) != 0;
                self.noise_halt = (data & 0x20) != 0;
            },
            0x400E => {
                match data & 0x0F {
                    0 => self.noise_seq.reload = 0x000,
                    1 => self.noise_seq.reload = 0x004,
                    2 => self.noise_seq.reload = 0x008,
                    3 => self.noise_seq.reload = 0x010,
                    4 => self.noise_seq.reload = 0x020,
                    5 => self.noise_seq.reload = 0x040,
                    6 => self.noise_seq.reload = 0x060,
                    7 => self.noise_seq.reload = 0x080,
                    8 => self.noise_seq.reload = 0x0A0,
                    9 => self.noise_seq.reload = 0x0CA,
                    10 => self.noise_seq.reload = 0x0FE,
                    11 => self.noise_seq.reload = 0x17C,
                    12 => self.noise_seq.reload = 0x1FC,
                    13 => self.noise_seq.reload = 0x3F8,
                    14 => self.noise_seq.reload = 0x7F2,
                    15 => self.noise_seq.reload = 0xFE4,
                    _ => ()
                }
            },
            0x4015 => {
                self.pulse1_enable = (data & 0x01) != 0;
                self.pulse2_enable = (data & 0x02) != 0;
                self.noise_enable = (data & 0x04) != 0;
                // println!("Pulse1 enable: {} - {}", self.pulse1_enable, data);
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

        self.global_time += 0.3333333333 / 1789773.0;

        if self.clock_counter % 6 == 0 {
            self.frame_clock_counter += 1;

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
                
                14916 => {
                    quarter_frame_clock = true;
                    half_frame_clock = true;
                    self.frame_clock_counter = 0;
                }

                _ => ()
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

            self.pulse1_osc.frequency = 1789773.0 / (16.0 * (self.pulse1_seq.reload as f32 + 1.0));
            self.pulse1_osc.amplitude = (self.pulse1_env.volume as f32 - 1.0) / 16.0;
            self.pulse1_sample = self.pulse1_osc.sample(self.global_time);

            if self.pulse1_lc.counter > 0 && self.pulse1_seq.timer >= 8 && !self.pulse1_sweep.mute && self.pulse1_env.output > 2 {
                self.pulse1_output += (self.pulse1_sample - self.pulse1_output) * 0.5;
            } else {
                self.pulse1_output = 0.0;
            }

            self.pulse2_seq.clock(self.pulse2_enable, |s| {
                *s = ((*s & 0x0001) << 7) | ((*s & 0x00FE) >> 1);
            });

            self.pulse2_osc.frequency = 1789773.0 / (16.0 * (self.pulse2_seq.reload as f32 + 1.0));
            self.pulse2_osc.amplitude = (self.pulse2_env.volume as f32 - 1.0) / 16.0;
            self.pulse2_sample = self.pulse2_osc.sample(self.global_time);

            if self.pulse2_lc.counter > 0 && self.pulse2_seq.timer >= 8 && !self.pulse2_sweep.mute && self.pulse2_env.output > 2 {
                self.pulse2_output += (self.pulse2_sample - self.pulse2_output) * 0.5;
            } else {
                self.pulse2_output = 0.0;
            }

            self.noise_seq.clock(self.noise_enable, |s| {
                *s = (((*s & 0x0001) ^ ((*s & 0x0002) >> 1)) << 14) | ((*s & 0x7FFF) >> 1);
            });

            if self.noise_lc.counter > 0 && self.noise_seq.timer >= 8 {
                self.noise_output = self.noise_seq.output as f32 * (self.noise_output - 1.0) as f32 / 16.0
            }

            if !self.pulse1_enable {self.pulse1_output = 0.0};
            if !self.pulse2_enable {self.pulse2_output = 0.0};
            if !self.noise_enable {self.noise_output = 0.0};
        }

        self.pulse1_sweep.track(self.pulse1_seq.reload);
        self.pulse2_sweep.track(self.pulse2_seq.reload);
        self.clock_counter = self.clock_counter.wrapping_add(1);

    }

    pub fn reset(&mut self) {
    }

    pub fn get_output_sample(&mut self) -> f32 {
        ((1.0 * self.pulse1_output) - 0.8) * 0.1 +
        ((1.0 * self.pulse2_output) - 0.8) * 0.1 +
        ((2.0 * self.noise_output) - 0.5) * 0.1
    }
}