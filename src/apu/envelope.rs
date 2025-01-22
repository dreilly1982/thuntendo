pub struct Envelope {
    pub start: bool,
    pub disable: bool,
    pub divider_count: u16,
    pub volume: u16,
    pub output: u16,
    pub decay_count: u16,
}

impl Envelope {
    pub fn new() -> Envelope {
        Envelope {
            start: false,
            disable: false,
            divider_count: 0,
            volume: 0,
            output: 0,
            decay_count: 0,
        }
    }

    pub fn clock(&mut self, should_loop: bool) {
        if !self.start {
            if self.divider_count == 0 {
                self.divider_count = self.volume;

                if self.decay_count == 0 {
                    if should_loop {
                        self.decay_count = 15;
                    }
                } else {
                    self.decay_count -= 1;
                }
            } else {
                self.divider_count -= 1;
            }
        } else {
            self.start = false;
            self.decay_count = 15;
            self.divider_count = self.volume;
        }

        if self.disable {
            self.output = self.volume;
        } else {
            self.output = self.decay_count;
        }
    }
}