use std::f32::consts::PI;

pub struct OscillatorPulse {
    pub frequency: f32,
    pub duty_cycle: f32,
    pub amplitude: f32,
    pub harmonics: f32,
}

impl OscillatorPulse {
    pub fn new() -> OscillatorPulse {
        OscillatorPulse {
            frequency: 0.0,
            duty_cycle: 0.0,
            amplitude: 1.0,
            harmonics: 20.0,
        }
    }

    pub fn sample(&self, t: f32) -> f32 {
        let mut a: f32 = 0.0;
        let mut b: f32 = 0.0;
        let p: f32 = self.duty_cycle * 2.0 * PI;

        let approxsin = |t: f32| {
            let mut j = t * 0.15915;
            j = j - j.floor();
            20.785 * j * (j - 0.5) * (j - 1.0)
        };

        for i in 1..self.harmonics as i32 {
            let n = i as f32;
            let c = n * self.frequency * 2.0 * PI * t;
            a += -approxsin(c) / n;
            b += -approxsin(c - p * n) / n;
        }
        
        let ret_val = (2.0 * self.amplitude / PI) * (a - b);
        ret_val
    }
}