use std::cell::RefCell;
use std::rc::Rc;
use rand::distributions::uniform::SampleBorrow;

use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::apu::APU;

const NTSC_CRYSTAL_FREQUENCY: f64 = 5369318.0;

pub struct NES<'a> {
    pub ram: RefCell<[u8; 2048]>,
    pub cpu: RefCell<Option<Rc<RefCell<CPU<'a>>>>>,
    pub cart: RefCell<Option<Rc<RefCell<Cartridge>>>>,
    pub ppu: RefCell<Option<Rc<RefCell<PPU<'a>>>>>,
    pub apu: RefCell<Option<Rc<RefCell<APU>>>>,
    pub controller: RefCell<[u8; 2]>,
    pub controller_state: RefCell<[u8; 2]>,
    pub dma_page: RefCell<u8>,
    pub dma_addr: RefCell<u8>,
    pub dma_data: RefCell<u8>,
    pub dma_dummy: RefCell<bool>,
    pub dma_transfer: RefCell<bool>,
    pub audio_sample: RefCell<f64>,
    pub audio_sample_ready: RefCell<bool>,
    audio_time_per_system_sample: RefCell<f64>,
    audio_time_per_nes_clock: RefCell<f64>,
    audio_time: RefCell<f64>,
}

impl<'a> NES<'a> {
    pub fn new() -> Rc<Self> {
        let bus = Rc::new(NES {
            ram: RefCell::new([0; 2048]),
            cpu: RefCell::new(None),
            cart: RefCell::new(None),
            ppu: RefCell::new(None),
            apu: RefCell::new(None),
            controller: RefCell::new([0; 2]),
            controller_state: RefCell::new([0; 2]),
            dma_page: RefCell::new(0x00),
            dma_addr: RefCell::new(0x00),
            dma_data: RefCell::new(0x00),
            dma_dummy: RefCell::new(true),
            dma_transfer: RefCell::new(false),
            audio_sample: RefCell::new(0.0),
            audio_time_per_system_sample: RefCell::new(0.0),
            audio_time_per_nes_clock: RefCell::new(0.0),
            audio_time: RefCell::new(0.0),
            audio_sample_ready: RefCell::new(false),
        });

        let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&bus))));
        let ppu = Rc::new(RefCell::new(PPU::new(Rc::clone(&bus))));
        let apu = Rc::new(RefCell::new(APU::new()));

        *bus.cpu.borrow_mut() = Some(Rc::clone(&cpu));
        *bus.ppu.borrow_mut() = Some(Rc::clone(&ppu));
        *bus.apu.borrow_mut() = Some(Rc::clone(&apu));

        bus
    }

    pub fn cpu_write(&self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.ram.borrow_mut()[addr as usize & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            if let Some(ppu) = self.ppu.borrow().as_ref() {
                ppu.borrow_mut().cpu_write(addr & 0x0007, data);
            }
        } else if (addr >= 0x4000 && addr <= 0x4013) || addr == 0x4015 || addr == 0x4017 {
            if let Some(apu) = self.apu.borrow().as_ref() {
                apu.borrow_mut().cpu_write(addr, data);
            }
        } else if addr == 0x4014 {
            *self.dma_page.borrow_mut() = data;
            *self.dma_addr.borrow_mut() = 0x00;
            *self.dma_transfer.borrow_mut() = true;
        } else if addr == 0x4016 {
            *self.controller_state.borrow_mut() = *self.controller.borrow();
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
        } else if addr == 0x4016 || addr == 0x4017 {
            data = (self.controller_state.borrow()[(addr & 0x0001) as usize] & 0x80 > 0) as u8;
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
        if let Some(apu) = self.apu.borrow().as_ref() {
            apu.borrow_mut().clock();
        }
        if *sys_counter % 3 == 0 {
            if *self.dma_transfer.borrow() {
                if *self.dma_dummy.borrow() {
                    if *sys_counter % 2 == 1 {
                        *self.dma_dummy.borrow_mut() = false;
                    }
                } else {
                    if *sys_counter % 2 == 0 {
                        *self.dma_data.borrow_mut() = self.cpu_read((*self.dma_page.borrow() as u16) << 8 | *self.dma_addr.borrow() as u16);
                    } else {
                        if let Some(ppu) = self.ppu.borrow().as_ref() {
                            let mut ppu_mut = ppu.borrow_mut();
                            ppu_mut.oam_addr = *self.dma_addr.borrow();
                            ppu_mut.oam_write(*self.dma_data.borrow());
                            *self.dma_addr.borrow_mut() += 1;

                            if *self.dma_addr.borrow() == 0x00 {
                                *self.dma_transfer.borrow_mut() = false;
                                *self.dma_dummy.borrow_mut() = true;
                            }
                        }
                    }
                }
            }
            if let Some(cpu) = self.cpu.borrow().as_ref() {
                cpu.borrow_mut().clock();
            }
        }

        *self.audio_time.borrow_mut() += *self.audio_time_per_nes_clock.borrow();

        *self.audio_sample_ready.borrow_mut() = false;
        // println!("audio_time: {}, audio_time_per_system_sample: {}", *self.audio_time.borrow(), *self.audio_time_per_system_sample.borrow());
        if *self.audio_time.borrow_mut() >= *self.audio_time_per_system_sample.borrow() {
            *self.audio_time.borrow_mut() -= *self.audio_time_per_system_sample.borrow();
            if let Some(apu) = self.apu.borrow().as_ref() {
                *self.audio_sample.borrow_mut() = apu.borrow().get_output_sample();
            }
            *self.audio_sample_ready.borrow_mut() = true;
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

    // pub fn cpu_complete(&self) -> bool {
    //     if let Some(cpu) = self.cpu.borrow().as_ref() {
    //         cpu.borrow().complete()
    //     } else {
    //         false
    //     }
    // }

    pub fn insert_cartridge(&self, fname: &str) {
        let cart = Rc::new(RefCell::new(Cartridge::new(fname)));
        *self.cart.borrow_mut() = Some(Rc::clone(&cart));
        if let Some(ppu) = self.ppu.borrow().as_ref() {
            ppu.borrow_mut().insert_cartridge(cart);
        }
    }

    pub fn set_sample_frequency(&self, sample_rate: u32) {
        *self.audio_time_per_system_sample.borrow_mut() = 1.0 / (sample_rate as f64);
        *self.audio_time_per_nes_clock.borrow_mut() = 1.0 / NTSC_CRYSTAL_FREQUENCY; 
        // if let Some(apu) = self.apu.borrow().as_ref() {
        //     apu.borrow_mut().set_sample_frequency(freq);
        // }
    }
}
