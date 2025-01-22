pub struct Sweeper {
    pub enabled: bool,
    pub down: bool,
    pub reload: bool,
    pub shift: u8,
    pub period: u8,
    pub timer: u8,
    pub change: u16,
    pub mute: bool,
}

impl Sweeper {
    pub fn new() -> Sweeper {
        Sweeper {
            enabled: false,
            down: false,
            reload: false,
            shift: 0,
            period: 0,
            timer: 0,
            change: 0,
            mute: false,
        }
    }

    pub fn track(&mut self, target: u16) {
        if self.enabled {
            self.change = target >> self.shift;
            self.mute = (target < 8) || (target > 0x7FF);
        }
    }

    pub fn clock(&mut self, target: &mut u16, channel: bool) -> bool {
        let mut changed = false;

        if (self.timer == 0) && self.enabled && self.shift > 0 && !self.mute {
            if *target >= 8 && self.change <= 0x7FF {
                if self.down {
                    *target -= self.change - channel as u16;
                } else {
                    *target += self.change;
                }

                changed = true;
            }
        }

        if self.timer == 0 || self.reload {
            self.timer = self.period;
            self.reload = false;
        } else {
            self.timer -= 1;
        }

        self.mute = (*target < 8) || (*target > 0x7FF);

        changed
    }
}