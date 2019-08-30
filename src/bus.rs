use std::sync::RwLock;

pub struct Bus {
    pub ram: RwLock<[u8; 64 * 1024]>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: RwLock::new([0; 64 * 1024]),
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        if addr >= 0x0000 && addr <= 0xFFFF {
            let mut lock = self.ram.write().unwrap();
            lock[addr as usize] = data;
        }
    }

    pub fn read(&self, addr: u16, read_only: bool) -> u8 {
        if addr >= 0x00 && addr <= 0xFFFF {
            let lock = self.ram.read().unwrap();
            return lock[addr as usize];
        }
        0x00
    }
}
