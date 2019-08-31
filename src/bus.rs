use std::cell::RefCell;

pub struct Bus {
    pub ram: RefCell<[u8; 64 * 1024]>,
    pub vram: RefCell<[u8; 64 * 1024]>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            ram: RefCell::new([0; 64 * 1024]),
            vram: RefCell::new([0; 64 * 1024]),
        }
    }

    pub fn write(&self, addr: u16, data: u8) {
        let mut lock = self.ram.borrow_mut();
        lock[addr as usize] = data;
    }

    pub fn read(&self, addr: u16) -> u8 {
        let lock = self.ram.borrow();
        return lock[addr as usize];
    }
}
