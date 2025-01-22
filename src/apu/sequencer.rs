pub struct Sequencer {
    pub sequence: u32,
    pub new_sequence: u32,
    pub timer: u16,
    pub reload: u16,
    pub output: u8,
}

impl Sequencer {
    pub fn new() -> Sequencer {
        Sequencer {
            sequence: 0,
            new_sequence: 0,
            timer: 0,
            reload: 0,
            output: 0,
        }
    }

    pub fn clock<T>(&mut self, enable: bool, mut manip: T)
        where T: FnMut(&mut u32) 
    {
        if enable {
            self.timer -= 1;
            if self.timer == 0xFFFF {
                self.timer = self.reload;
                manip(&mut self.sequence);
                self.output = (self.sequence & 0x1) as u8;
            }
        }
    }
}