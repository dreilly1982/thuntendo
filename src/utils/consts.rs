pub const WIDTH: u32 = 256;
pub const HEIGHT: u32 = 240;
pub const NTSC_CRYSTAL_FREQUENCY: f32 = 5369318.0 / 2.0;

pub const LENGTH_TABLE: [u8; 32] = [
    10,  254, 20,  2,   40,  4,   80,  6,
    160, 8,   60,  10,  14,  12,  26,  14,
    12,  16,  24,  18,  48,  20,  96,  22,
    192, 24,  72,  26,  16,  28,  32,  30
];