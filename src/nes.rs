use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::rc::Rc;
use std::sync::Arc;
use ringbuf::{Producer, SharedRb};

use crate::cartridge::Cartridge;
use crate::cpu::CPU;
use crate::ppu::PPU;
use crate::apu::APU;
use crate::utils::consts::NTSC_CRYSTAL_FREQUENCY;

pub struct NES<'a> {
    pub ram: RefCell<[u8; 2048]>,
    pub cpu: RefCell<Option<Rc<RefCell<CPU<'a>>>>>,
    pub cart: RefCell<Option<Rc<RefCell<Cartridge>>>>,
    pub ppu: RefCell<PPU>,
    pub apu: RefCell<Option<Rc<RefCell<APU>>>>,
    pub controller: RefCell<[u8; 2]>,
    pub controller_state: RefCell<[u8; 2]>,
    pub dma_page: RefCell<u8>,
    pub dma_addr: RefCell<u8>,
    pub dma_data: RefCell<u8>,
    pub dma_dummy: RefCell<bool>,
    pub dma_transfer: RefCell<bool>,
    pub audio_sample_ready: RefCell<bool>,
    cycles_per_sample: RefCell<u32>,
    audio_time: RefCell<f32>,
    time_per_system_sample: RefCell<f32>,
    time_per_nes_clock: RefCell<f32>,
    audio_cycles: RefCell<u32>,
    sys_counter: RefCell<u32>,
}

impl<'a> NES<'a> {
    pub fn new() -> Rc<Self> {
        let bus = Rc::new(NES {
            ram: RefCell::new([0; 2048]),
            cpu: RefCell::new(None),
            cart: RefCell::new(None),
            ppu: RefCell::new(PPU::new()),
            apu: RefCell::new(None),
            controller: RefCell::new([0; 2]),
            controller_state: RefCell::new([0; 2]),
            dma_page: RefCell::new(0x00),
            dma_addr: RefCell::new(0x00),
            dma_data: RefCell::new(0x00),
            dma_dummy: RefCell::new(true),
            dma_transfer: RefCell::new(false),
            cycles_per_sample: RefCell::new(0),
            audio_time: RefCell::new(0.0),
            time_per_system_sample: RefCell::new(0.0),
            time_per_nes_clock: RefCell::new(0.0),
            audio_cycles: RefCell::new(0),
            audio_sample_ready: RefCell::new(false),
            sys_counter: RefCell::new(0),
        });

        let cpu = Rc::new(RefCell::new(CPU::new(Rc::clone(&bus))));
        let apu = Rc::new(RefCell::new(APU::new()));

        *bus.cpu.borrow_mut() = Some(Rc::clone(&cpu));
        *bus.apu.borrow_mut() = Some(Rc::clone(&apu));

        bus
    }

    pub fn cpu_write(&self, addr: u16, data: u8) {
        if addr <= 0x1FFF {
            self.ram.borrow_mut()[addr as usize & 0x07FF] = data;
        } else if addr >= 0x2000 && addr <= 0x3FFF {
            self.ppu.borrow_mut().cpu_write(addr & 0x0007, data);
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
            self.ppu.borrow_mut().cpu_read(addr & 0x0007)
        } else if addr == 0x4015 { 
            println!("Reading from 0x4015");
            data
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

    pub fn reset(&self) {
        if let Some(cpu) = self.cpu.borrow().as_ref() {
            cpu.borrow_mut().reset();
        }
        self.ppu.borrow_mut().reset();
        if let Some(apu) = self.apu.borrow().as_ref() {
            apu.borrow_mut().reset();
        }
        if let Some(cart) = self.cart.borrow().as_ref() {
            cart.borrow_mut().reset();
        }
        *self.sys_counter.borrow_mut() = 0;
    }

    pub fn get_frame_complete(&self) -> bool {
        self.ppu.borrow().get_frame_complete()
    }

    pub fn reset_frame_complete(&self) {
        self.ppu.borrow_mut().reset_frame_complete();
    }

    pub fn clock(&self, frame: &mut[u8], producer: &mut Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>) -> bool {
        self.ppu.borrow_mut().clock(frame);
        

        if let Some(apu) = self.apu.borrow().as_ref() {
            apu.borrow_mut().clock();
        }
        

        if *self.sys_counter.borrow() % 3 == 0 {
            if *self.dma_transfer.borrow() {
                if *self.dma_dummy.borrow() {
                    if *self.sys_counter.borrow() % 2 == 1 {
                        *self.dma_dummy.borrow_mut() = false;
                    }
                } else {
                    if *self.sys_counter.borrow() % 2 == 0 {
                        *self.dma_data.borrow_mut() = self.cpu_read((*self.dma_page.borrow() as u16) << 8 | *self.dma_addr.borrow() as u16);
                    } else {
                        let mut ppu_mut = self.ppu.borrow_mut();
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
            if let Some(cpu) = self.cpu.borrow().as_ref() {
                cpu.borrow_mut().clock();
            }
            
        }

        

        *self.audio_sample_ready.borrow_mut() = false;
        *self.audio_time.borrow_mut() += *self.time_per_nes_clock.borrow();
        if *self.audio_time.borrow() >= *self.time_per_system_sample.borrow() {
            *self.audio_time.borrow_mut() -= *self.time_per_system_sample.borrow();
            if let Some(apu) = self.apu.borrow().as_ref() {
                let sample = apu.borrow_mut().get_output_sample();
                while producer.push(sample).is_err() {
                    std::thread::sleep(std::time::Duration::from_micros(1));
                };
            }
            *self.audio_sample_ready.borrow_mut() = true;
            *self.audio_cycles.borrow_mut() -= *self.cycles_per_sample.borrow();
        }

        let mut ppu_mut = self.ppu.borrow_mut();
        if ppu_mut.nmi {
            ppu_mut.nmi = false;
            if let Some(cpu) = self.cpu.borrow().as_ref() {
                cpu.borrow_mut().nmi();
            }
            ppu_mut.status.remove(crate::ppu::Status::VERTICAL_BLANK);
        }

        *self.sys_counter.borrow_mut() += 1;
        *self.audio_sample_ready.borrow()
    }


    pub fn insert_cartridge(&self, fname: &str) {
        let cart = Rc::new(RefCell::new(Cartridge::new(fname)));
        *self.cart.borrow_mut() = Some(Rc::clone(&cart));
        self.ppu.borrow_mut().insert_cartridge(cart);
    }

    pub fn set_sample_frequency(&self, sample_rate: u32) {
        println!("Setting sample rate to: {}", sample_rate);
        *self.time_per_system_sample.borrow_mut() = 1.0 / sample_rate as f32;
        *self.time_per_nes_clock.borrow_mut() = 1.0 / NTSC_CRYSTAL_FREQUENCY as f32;
        if let Some(apu) = self.apu.borrow().as_ref() {
            apu.borrow_mut().audio_sample_rate = sample_rate as f32;
        }
    }
}
