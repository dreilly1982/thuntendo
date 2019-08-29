pub struct Bus {
    pub ram: [u8; 64 * 1024],
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: [0; 64 * 1024],
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        }
    }

    pub fn read(&self, addr: u16, read_only: bool) -> u8 {
        if addr >= 0x00 && addr <= 0xFFFF {
            return self.ram[addr as usize];
        }
        0x00
    }
}
