use std::cell::RefCell;
use std::rc::Rc;
use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::{self, PPU};

pub struct NES<'a> {
    pub ram: RefCell<[u8; 2048]>,
    pub cpu: RefCell<Option<Rc<RefCell<CPU<'a>>>>>,
    pub cart: RefCell<Option<Rc<RefCell<Cartridge>>>>,
    pub ppu: RefCell<Option<Rc<RefCell<PPU<'a>>>>>,
    pub controller: RefCell<[u8; 2]>,
    pub controller_state: RefCell<[u8; 2]>,
}

impl<'a> NES<'a> {
    pub fn new() -> Rc<Self> {
        let bus = Rc::new(NES {
            ram: RefCell::new([0; 2048]),
            cpu: RefCell::new(None),
            cart: RefCell::new(None),
            ppu: RefCell::new(None),
            controller: RefCell::new([0; 2]),
            controller_state: RefCell::new([0; 2]),
        });

        let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&bus))));
        let ppu = Rc::new(RefCell::new(PPU::new(Rc::clone(&bus))));

        *bus.cpu.borrow_mut() = Some(Rc::clone(&cpu));
        *bus.ppu.borrow_mut() = Some(Rc::clone(&ppu));

        bus
    }

    pub fn cpu_write(&self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.ram.borrow_mut()[addr as usize & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            if let Some(ppu) = self.ppu.borrow().as_ref() {
                ppu.borrow_mut().cpu_write(addr & 0x0007, data);
            }
        } else if addr == 0x4016 || addr == 0x4017 {
            self.controller_state.borrow_mut()[(addr & 0x0001) as usize] = self.controller.borrow()[(addr & 0x0001) as usize];
        } else if addr >= 0x8000 {
            if let Some(cart) = self.cart.borrow().as_ref() {
                cart.borrow_mut().cpu_write(addr, data);
            }
        }
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        let mut data: u8 = 0x00;
        if addr <= 0x1FFF {
            self.ram.borrow()[addr as usize & 0x07FF]
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            if let Some(ppu) = self.ppu.borrow().as_ref() {
                ppu.borrow_mut().cpu_read(addr & 0x0007)
            } else {
                data
            }
        } else if addr == 4016 || addr == 4017 {
            if self.controller_state.borrow()[(addr & 0x0001) as usize] & 0x80 > 0 {
                data = 0x01;
            } else {
                data = 0x00;
            }
            self.controller_state.borrow_mut()[(addr & 0x0001) as usize] <<= 1;
            data
        } else if addr >= 0x8000 {
            if let Some(cart) = self.cart.borrow().as_ref() {
                cart.borrow().cpu_read(addr)
            } else {
                data
            }
        } else {
            data
        }
    }

    pub fn reset(&self, sys_counter: &mut u32) {
        if let Some(cpu) = self.cpu.borrow().as_ref() {
            cpu.borrow_mut().reset();
        }
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow_mut().reset();
        }
        *sys_counter = 0;
    }

    pub fn get_frame_complete(&self) -> bool {
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow().get_frame_complete()
        } else {
            false
        }
    }

    pub fn reset_frame_complete(&self) {
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow_mut().reset_frame_complete();
        }
    }

    pub fn clock(&self, frame: &mut[u8], sys_counter: &mut u32) {
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow_mut().clock(frame);
        }
        if *sys_counter % 3 == 0 {
            if let Some(cpu) = self.cpu.borrow().as_ref() {
                cpu.borrow_mut().clock();
            }
        }

        if let Some(ppu) = self.ppu.borrow().as_ref() {
            if let Ok(mut ppu_mut) = ppu.try_borrow_mut() {
                if ppu_mut.nmi {
                    ppu_mut.nmi = false;
                    if let Some(cpu) = self.cpu.borrow().as_ref() {
                        cpu.borrow_mut().nmi();
                }
            }
            }
        }
        *sys_counter += 1;
    }

    pub fn cpu_complete(&self) -> bool {
        if let Some(cpu) = self.cpu.borrow().as_ref() {
            cpu.borrow().complete()
        } else {
            false
        }
    }

    pub fn insert_cartridge(&self, fname: &str) {
        let cart = Rc::new(RefCell::new(Cartridge::new(fname)));
        *self.cart.borrow_mut() = Some(Rc::clone(&cart));
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow_mut().insert_cartridge(cart);
        }
        // self.cpu_write(0xFFFC, 0x00);
        // self.cpu_write(0xFFFD, 0xC0);
    }
}
