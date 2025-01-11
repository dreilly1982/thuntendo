pub const LENGTH_TABLE: [u8; 32] = [
    10,  254, 20,  2,   40,  4,   80,  6,
    160, 8,   60,  10,  14,  12,  26,  14,
    12,  16,  24,  18,  48,  20,  96,  22,
    192, 24,  72,  26,  16,  28,  32,  30
];

pub struct APU {
    pulse1_enable: bool,
    pulse1_sample: f64,
}

impl APU {
    pub fn new() -> APU {
        APU {
            pulse1_enable: false,
            pulse1_sample: 0.0,
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        0
    }

    pub fn clock(&mut self) {
    }

    pub fn reset(&mut self) {
    }

    pub fn get_output_sample(&self) -> f64 {
        0.0
    }
}