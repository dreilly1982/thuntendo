use std::sync::RwLock;

pub struct Bus {
    pub ram: RwLock<[u8; 64 * 1024]>,
    pub vram: RwLock<[u8; 64 * 1024]>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: RwLock::new([0; 64 * 1024]),
            vram: RwLock::new([0; 64 * 1024]),
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        let mut lock = self.ram.write().unwrap();
        lock[addr as usize] = data;
    }

    pub fn read(&self, addr: u16) -> u8 {
        let lock = self.ram.read().unwrap();
        return lock[addr as usize];
    }

    pub fn write_vram(&self, addr: u16, data: u8) {
        let mut lock = self.vram.write().unwrap();
        lock[addr as usize] = data;
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        let lock = self.vram.read().unwrap();
        return lock[addr as usize];
    }
}
