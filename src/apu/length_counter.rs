pub struct LengthCounter {
    pub counter: u8,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            counter: 0,
        }
    }

    pub fn clock(&mut self, enable: bool, halt: bool) -> u8 {

        if !enable {
            self.counter = 0;
        } else if !halt && self.counter > 0 {
            self.counter -= 1;
        }

        self.counter
    }
}