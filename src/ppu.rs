use crate::nes::NES;
use crate::cartridge::{Cartridge, MIRROR};
use crate::WIDTH;
use crate::HEIGHT;
use std::cell::RefCell;
use std::os::macos::raw::stat;
use std::panic;
use std::rc::Rc;
use rand::Rng;
use bitflags::bitflags;

bitflags! {
    pub struct Status: u8 {
        const VERTICAL_BLANK = 0b1000_0000;
        const SPRITE_ZERO_HIT = 0b0100_0000;
        const SPRITE_OVERFLOW = 0b0010_0000;
    }
}

bitflags! {
    pub struct Mask: u8 {
        const ENHANCE_BLUE = 0b1000_0000;
        const ENHANCE_GREEN = 0b0100_0000;
        const ENHANCE_RED = 0b0010_0000;
        const RENDER_SPRITES = 0b0001_0000;
        const RENDER_BACKGROUND = 0b0000_1000;
        const RENDER_SPRITES_LEFT = 0b0000_0100;
        const RENDER_BACKGROUND_LEFT = 0b0000_0010;
        const GRAYSCALE = 0b0000_0001;
    }
}

bitflags! {
    pub struct PPUCTRL: u8 {
        const NAMETABLE_X = 0b0000_0001;
        const NAMETABLE_Y = 0b0000_0010;
        const INCREMENT_MODE = 0b0000_0100;
        const PATTERN_SPRITE = 0b0000_1000;
        const PATTERN_BACKGROUND = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const SLAVE_MODE = 0b0100_0000;
        const ENABLE_NMI = 0b1000_0000;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LoopyRegister {
    reg: u16,
}

impl LoopyRegister {
    pub fn new() -> Self {
        LoopyRegister { reg: 0x0000 }
    }

    // Getters and setters for each field
    pub fn coarse_x(&self) -> u16 { self.reg & 0x001F }
    pub fn set_coarse_x(&mut self, value: u16) { self.reg = (self.reg & !0x001F) | (value & 0x001F); }

    pub fn coarse_y(&self) -> u16 { (self.reg >> 5) & 0x001F }
    pub fn set_coarse_y(&mut self, value: u16) { self.reg = (self.reg & !0x03E0) | ((value & 0x001F) << 5); }

    pub fn nametable_x(&self) -> bool { (self.reg & 0x0400) != 0 }
    pub fn set_nametable_x(&mut self, value: bool) { self.reg = (self.reg & !0x0400) | (if value { 0x0400 } else { 0 }); }

    pub fn nametable_y(&self) -> bool { (self.reg & 0x0800) != 0 }
    pub fn set_nametable_y(&mut self, value: bool) { self.reg = (self.reg & !0x0800) | (if value { 0x0800 } else { 0 }); }

    pub fn fine_y(&self) -> u16 { (self.reg >> 12) & 0x0007 }
    pub fn set_fine_y(&mut self, value: u16) { self.reg = (self.reg & !0x7000) | ((value & 0x0007) << 12); }

    pub fn unused(&self) -> bool { (self.reg & 0x8000) != 0 }
    pub fn set_unused(&mut self, value: bool) { self.reg = (self.reg & !0x8000) | (if value { 0x8000 } else { 0 }); }

    // Raw register access
    pub fn bits(&self) -> u16 { self.reg }
    pub fn set_bits(&mut self, value: u16) { self.reg = value; }
}

pub struct PPU<'a> {
    pub bus: Rc<NES<'a>>,
    pub cart: RefCell<Option<Rc<RefCell<Cartridge>>>>,
    table_name: [[u8; 1024]; 2],
    table_pattern: [[u8; 4096]; 2],
    table_palette: [u8; 32],
    screen_palette: [[u8; 4]; 0x40],
    cycle: i16,
    scanline: i16,
    frame_complete: bool,
    fine_x: u8,
    address_latch: u8,
    ppu_data_buffer: u8,
    bg_next_tile_id: u8,
    bg_next_tile_attrib: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,
    bg_shifter_pattern_lo: u16,
    bg_shifter_pattern_hi: u16,
    bg_shifter_attrib_lo: u16,
    bg_shifter_attrib_hi: u16,
    status: Status,
    mask: Mask,
    control: PPUCTRL,
    vram_addr: LoopyRegister,
    tram_addr: LoopyRegister,
    pub nmi: bool,
}

impl<'a> PPU<'a> {
    pub fn new(b: Rc<NES<'a>>) -> PPU<'a> {
        let screen_palette: [[u8; 4]; 0x40] = [
            [84, 84, 84, 255],
            [0, 30, 116, 255],
            [8, 16, 144, 255],
            [48, 0, 136, 255],
            [68, 0, 100, 255],
            [92, 0, 48, 255],
            [84, 4, 0, 255],
            [60, 24, 0, 255],
            [32, 42, 0, 255],
            [8, 58, 0, 255],
            [0, 64, 0, 255],
            [0, 60, 0, 255],
            [0, 50, 60, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],

            [152, 150, 152, 255],
            [8, 76, 196, 255],
            [48, 50, 236, 255],
            [92, 30, 228, 255],
            [136, 20, 176, 255],
            [160, 20, 100, 255],
            [152, 34, 32, 255],
            [120, 60, 0, 255],
            [84, 90, 0, 255],
            [40, 114, 0, 255],
            [8, 124, 0, 255],
            [0, 118, 40, 255],
            [0, 102, 120, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],

            [236, 238, 236, 255],
            [76, 154, 236, 255],
            [120, 124, 236, 255],
            [176, 98, 236, 255],
            [228, 84, 236, 255],
            [236, 88, 180, 255],
            [236, 106, 100, 255],
            [212, 136, 32, 255],
            [160, 170, 0, 255],
            [116, 196, 0, 255],
            [76, 208, 32, 255],
            [56, 204, 108, 255],
            [56, 180, 204, 255],
            [60, 60, 60, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255],

            [236, 238, 236, 255],
            [168, 204, 236, 255],
            [188, 188, 236, 255],
            [212, 178, 236, 255],
            [236, 174, 236, 255],
            [236, 174, 212, 255],
            [236, 180, 176, 255],
            [228, 196, 144, 255],
            [204, 210, 120, 255],
            [180, 222, 120, 255],
            [168, 226, 144, 255],
            [152, 226, 180, 255],
            [160, 214, 228, 255],
            [160, 162, 160, 255],
            [0, 0, 0, 255],
            [0, 0, 0, 255]];
        PPU {
            bus: b,
            cart: RefCell::new(None),
            table_name: [[0; 1024]; 2],
            table_pattern: [[0; 4096]; 2],
            table_palette: [0; 32],
            screen_palette: screen_palette,
            cycle: 0,
            scanline: 0,
            frame_complete: false,
            fine_x: 0x00,
            address_latch: 0x00,
            ppu_data_buffer: 0x00,
            bg_next_tile_id: 0x00,
            bg_next_tile_attrib: 0x00,
            bg_next_tile_lsb: 0x00,
            bg_next_tile_msb: 0x00,
            bg_shifter_pattern_lo: 0x0000,
            bg_shifter_pattern_hi: 0x0000,
            bg_shifter_attrib_lo: 0x0000,
            bg_shifter_attrib_hi: 0x0000,
            status: Status::empty(),
            mask: Mask::empty(),
            control: PPUCTRL::empty(),
            vram_addr: LoopyRegister::new(),
            tram_addr: LoopyRegister::new(),
            nmi: false,
        }
    }

    pub fn insert_cartridge(&mut self, cart: Rc<RefCell<Cartridge>>) {
        self.cart = RefCell::new(Some(cart));
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        let mut data: u8 = 0x00;

        match addr {
            0x0000 => (),
            0x0001 => (),
            0x0002 => {
                data = (self.status.bits() & 0xE0) | (self.ppu_data_buffer & 0x1F);
                self.status.remove(Status::VERTICAL_BLANK);
                self.address_latch = 0x00;
            },
            0x0003 => (),
            0x0004 => (),
            0x0005 => (),
            0x0006 => (),
            0x0007 => {
                data = self.ppu_data_buffer;
                self.ppu_data_buffer = self.ppu_read(self.vram_addr.bits());
                if self.vram_addr.reg >= 0x3F00 {
                    data = self.ppu_data_buffer;
                }
                self.vram_addr.set_bits(self.vram_addr.bits() + if self.control.contains(PPUCTRL::INCREMENT_MODE) { 32 } else { 1 });
            },
            _ => ()
        }
        data
    }

    pub fn cpu_write(&mut self, addr:u16, data: u8) {
        match addr {
            0x0000 => {
                self.control = PPUCTRL::from_bits_truncate(data);
                self.tram_addr.set_nametable_x(self.control.contains(PPUCTRL::NAMETABLE_X));
                self.tram_addr.set_nametable_y(self.control.contains(PPUCTRL::NAMETABLE_Y));
            },
            0x0001 => {
                self.mask = Mask::from_bits_truncate(data);
            },
            0x0002 => (),
            0x0003 => (),
            0x0004 => (),
            0x0005 => {
                if self.address_latch == 0 {
                    self.fine_x = data & 0x07;
                    self.tram_addr.set_coarse_x((data >> 3) as u16);
                    self.address_latch = 0x01;
                } else {
                    self.tram_addr.set_fine_y((data & 0x07) as u16);
                    self.tram_addr.set_coarse_y((data >> 3) as u16);
                    self.address_latch = 0x00;
                }
            },
            0x0006 => {
                if self.address_latch == 0 {
                    self.tram_addr.set_bits(((data as u16) & 0x3F) << 8 | (self.tram_addr.bits() & 0x00FF));
                    self.address_latch = 0x01;
                } else {
                    self.tram_addr.set_bits((self.tram_addr.bits() & 0xFF00) | data as u16);
                    self.vram_addr.set_bits(self.tram_addr.bits());
                    self.address_latch = 0x00;
                }
            },
            0x0007 => {
                self.ppu_write(self.vram_addr.bits(), data);
                self.vram_addr.set_bits(self.vram_addr.bits() + if self.control.contains(PPUCTRL::INCREMENT_MODE) { 32 } else { 1 });
            },
            _ => ()
        }
    }

    pub fn ppu_read(&self, mut addr: u16) -> u8 {
        let mut data: u8 = 0x00;
        addr &= 0x3FFF;
        if let Some(cart) = self.cart.borrow().as_ref() {
            if let Ok(value) = cart.borrow().ppu_read(addr) {
                data = value;
            } else if addr <= 0x1FFF {
                data = self.table_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0FFF) as usize];
            } else if addr >= 0x2000 && addr <= 0x3EFF {
                addr &= 0x0FFF;
                if cart.borrow().mirroring == MIRROR::VERTICAL {
                    if addr <= 0x03FF {
                        data = self.table_name[0][(addr & 0x3FF) as usize];
                    } else if addr >= 0x0400 && addr <= 0x07FF {
                        data = self.table_name[1][(addr & 0x03FF) as usize];
                    } else if addr >= 0x0800 && addr <= 0x0BFF {
                        data = self.table_name[0][(addr & 0x03FF) as usize];
                    } else if addr >= 0x0C00 && addr <= 0x0FFF {
                        data = self.table_name[1][(addr & 0x03FF) as usize];
                    }
                } else if cart.borrow().mirroring == MIRROR::HORIZONTAL {
                    if addr <= 0x03FF {
                        data = self.table_name[0][(addr & 0x3FF) as usize];
                    } else if addr >= 0x0400 && addr <= 0x07FF {
                        data = self.table_name[0][(addr & 0x03FF) as usize];
                    } else if addr >= 0x0800 && addr <= 0x0BFF {
                        data = self.table_name[1][(addr & 0x03FF) as usize];
                    } else if addr >= 0x0C00 && addr <= 0x0FFF {
                        data = self.table_name[1][(addr & 0x03FF) as usize];
                    }
                }
            } else if addr >= 0x3F00 && addr <= 0x3FFF {
                addr &= 0x001F;
                if addr == 0x0010 { addr = 0x0000; }
                if addr == 0x0014 { addr = 0x0004; }
                if addr == 0x0018 { addr = 0x0008; }
                if addr == 0x001C { addr = 0x000C; }
                if self.mask.contains(Mask::GRAYSCALE) {
                    data = self.table_palette[addr as usize] & 0x30;
                } else {
                    data = self.table_palette[addr as usize] & 0x3F;
                }
            }
        };
        data
    }

    pub fn ppu_write(&mut self, mut addr: u16, data: u8) {
        addr &= 0x3FFF;
        if let Some(cart) = self.cart.borrow().as_ref() {
            if let Ok(mut cart_mut) = cart.try_borrow_mut() {
                if let Ok(_) = cart_mut.ppu_write(addr, data) {
                    return;
                }
            }
            if addr <= 0x1FFF {
                self.table_pattern[((addr & 0x1000) >> 12) as usize][(addr & 0x0FFF) as usize] = data;
            } else if addr >= 0x2000 && addr <= 0x3EFF {
                addr &= 0x0FFF;
                if cart.borrow().mirroring == MIRROR::VERTICAL {
                    if addr <= 0x03FF {
                        self.table_name[0][(addr & 0x3FF) as usize] = data;
                    } else if addr >= 0x0400 && addr <= 0x07FF {
                        self.table_name[1][(addr & 0x03FF) as usize] = data;
                    } else if addr >= 0x0800 && addr <= 0x0BFF {
                        self.table_name[0][(addr & 0x03FF) as usize] = data;
                    } else if addr >= 0x0C00 && addr <= 0x0FFF {
                        self.table_name[1][(addr & 0x03FF) as usize] = data;
                    }
                } else if cart.borrow().mirroring == MIRROR::HORIZONTAL {
                    if addr <= 0x03FF {
                        self.table_name[0][(addr & 0x3FF) as usize] = data;
                    } else if addr >= 0x0400 && addr <= 0x07FF {
                        self.table_name[0][(addr & 0x03FF) as usize] = data;
                    } else if addr >= 0x0800 && addr <= 0x0BFF {
                        self.table_name[1][(addr & 0x03FF) as usize] = data;
                    } else if addr >= 0x0C00 && addr <= 0x0FFF {
                        self.table_name[1][(addr & 0x03FF) as usize] = data;
                    }
                }
            } else if addr >= 0x3F00 && addr <= 0x3FFF {
                addr &= 0x001F;
                if addr == 0x0010 { addr = 0x0000; }
                if addr == 0x0014 { addr = 0x0004; }
                if addr == 0x0018 { addr = 0x0008; }
                if addr == 0x001C { addr = 0x000C; }
                self.table_palette[addr as usize] = data;
            }
        };
    }

    pub fn get_frame_complete(&self) -> bool {
        self.frame_complete
    }

    pub fn reset_frame_complete(&mut self) {
        self.frame_complete = false;
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.scanline = 0;
        self.frame_complete = false;
        self.fine_x = 0x00;
        self.address_latch = 0x00;
        self.ppu_data_buffer = 0x00;
        self.bg_next_tile_id = 0x00;
        self.bg_next_tile_attrib = 0x00;
        self.bg_next_tile_lsb = 0x00;
        self.bg_next_tile_msb = 0x00;
        self.bg_shifter_pattern_lo = 0x0000;
        self.bg_shifter_pattern_hi = 0x0000;
        self.bg_shifter_attrib_lo = 0x0000;
        self.bg_shifter_attrib_hi = 0x0000;
        self.status = Status::from_bits_truncate(0x00);
        self.mask = Mask::from_bits_truncate(0x00);
        self.control = PPUCTRL::from_bits_truncate(0x00);
        self.vram_addr.set_bits(0x0000);
        self.tram_addr.set_bits(0x0000);
    }

    pub fn increment_scroll_x(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
            if self.vram_addr.coarse_x() == 31 {
                self.vram_addr.set_coarse_x(0x0000);
                self.vram_addr.reg ^= 0x0400;
            } else {
                self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
            }
        }
    }

    pub fn increment_scroll_y(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
            if self.vram_addr.fine_y() < 7 {
                self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
            } else {
                self.vram_addr.set_fine_y(0x0000);
                if self.vram_addr.coarse_y() == 29 {
                    self.vram_addr.set_coarse_y(0x0000);
                    self.vram_addr.reg ^= 0x0800;
                } else if self.vram_addr.coarse_y() == 31 {
                    self.vram_addr.set_coarse_y(0x00);
                } else {
                    self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
                }
            }
        }
    }

    pub fn transfer_address_x(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
            self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
            self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
        }
    }

    pub fn transfer_address_y(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
            self.vram_addr.set_fine_y(self.tram_addr.fine_y());
            self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
            self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
        }
    }

    pub fn load_background_shifters(&mut self) {
        self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | if (self.bg_next_tile_attrib & 0b01) != 0 { 0xFF } else { 0x00 };
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | if (self.bg_next_tile_attrib & 0b10) != 0 { 0xFF } else { 0x00 };
    }

    pub fn update_shifters(&mut self) {
        if self.mask.contains(Mask::RENDER_BACKGROUND) {
            self.bg_shifter_pattern_lo <<= 1;
            self.bg_shifter_pattern_hi <<= 1;

            self.bg_shifter_attrib_lo <<= 1;
            self.bg_shifter_attrib_hi <<= 1;
        }
    }

    pub fn clock(&mut self, frame: &mut [u8]) {

        // let mut increment_scroll_x = || {
        //     if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
        //         if self.vram_addr.coarse_x() == 31 {
        //             self.vram_addr.set_coarse_x(0x0000);
        //             self.vram_addr.reg ^= 0x0400;
        //         } else {
        //             self.vram_addr.set_coarse_x(self.vram_addr.coarse_x() + 1);
        //         }
        //     }
        // };

        // let mut increment_scroll_y = || {
        //     if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
        //         if self.vram_addr.fine_y() < 7 {
        //             self.vram_addr.set_fine_y(self.vram_addr.fine_y() + 1);
        //         } else {
        //             self.vram_addr.set_fine_y(0x0000);
        //             if self.vram_addr.coarse_y() == 29 {
        //                 self.vram_addr.set_coarse_y(0x0000);
        //                 self.vram_addr.reg ^= 0x0800;
        //             } else if self.vram_addr.coarse_y() == 31 {
        //                 self.vram_addr.set_coarse_y(0x00);
        //             } else {
        //                 self.vram_addr.set_coarse_y(self.vram_addr.coarse_y() + 1);
        //             }
        //         }
        //     }
        // };

        // let mut transfer_address_x = || {
        //     if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
        //         self.vram_addr.set_nametable_x(self.tram_addr.nametable_x());
        //         self.vram_addr.set_coarse_x(self.tram_addr.coarse_x());
        //     }
        // };

        // let mut transfer_address_y = || {
        //     if self.mask.contains(Mask::RENDER_BACKGROUND | Mask::RENDER_SPRITES) {
        //         self.vram_addr.set_fine_y(self.tram_addr.fine_y());
        //         self.vram_addr.set_nametable_y(self.tram_addr.nametable_y());
        //         self.vram_addr.set_coarse_y(self.tram_addr.coarse_y());
        //     }
        // };

        // let mut load_background_shifters = || {
        //     self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | self.bg_next_tile_lsb as u16;
        //     self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | self.bg_next_tile_msb as u16;

        //     let mut palette: u16 = 0x00;
        //     if self.mask.contains(Mask::RENDER_BACKGROUND) {
        //         palette = ((self.bg_next_tile_attrib as u16) >> ((self.vram_addr.coarse_x() & 0x02) | ((self.vram_addr.coarse_y() & 0x02) << 1))) & 0x03;
        //     }

        //     self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | ((palette & 0x01) << 0) as u16;
        //     self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | ((palette & 0x02) << 1) as u16;

        //     self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | if (self.bg_next_tile_attrib & 0b01) != 0 { 0xFF } else { 0x00 };
        //     self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | if (self.bg_next_tile_attrib & 0b10) != 0 { 0xFF } else { 0x00 };
        // };

        // let mut update_shifters = || {
        //     if self.mask.contains(Mask::RENDER_BACKGROUND) {
        //         self.bg_shifter_pattern_lo <<= 1;
        //         self.bg_shifter_pattern_hi <<= 1;

        //         self.bg_shifter_attrib_lo <<= 1;
        //         self.bg_shifter_attrib_hi <<= 1;
        //     }
        // };

        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.remove(Status::VERTICAL_BLANK);
                // self.status.remove(Status::SPRITE_ZERO_HIT);
                // self.status.remove(Status::SPRITE_OVERFLOW);
            }

            if (self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338) {
                self.update_shifters();

                match (self.cycle - 1) % 8 {
                    0 => {
                        self.load_background_shifters();
                        self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.bits() & 0x0FFF));
                    },
                    2 => {
                        self.bg_next_tile_attrib = self.ppu_read(0x23C0 | (self.vram_addr.nametable_y() as u16) << 11 
                                                                                | (self.vram_addr.nametable_x() as u16) << 10 
                                                                                | ((self.vram_addr.coarse_y() >> 2) << 3) 
                                                                                | (self.vram_addr.coarse_x() >> 2));
                        if (self.vram_addr.coarse_y() & 0x02) != 0 {
                            self.bg_next_tile_attrib >>= 4;
                        }

                        if (self.vram_addr.coarse_x() & 0x02) != 0 {
                            self.bg_next_tile_attrib >>= 2;
                        }

                        self.bg_next_tile_attrib &= 0x03;
                    },
                    4 => {
                        self.bg_next_tile_lsb = self.ppu_read(((self.control.contains(PPUCTRL::PATTERN_BACKGROUND) as u16) << 12) 
                                                                + ((self.bg_next_tile_id as u16) << 4) 
                                                                + (self.vram_addr.fine_y() as u16) + 0);
                    },
                    6 => {
                        self.bg_next_tile_msb = self.ppu_read(((self.control.contains(PPUCTRL::PATTERN_BACKGROUND) as u16) << 12) 
                                                                + ((self.bg_next_tile_id as u16) << 4) 
                                                                + (self.vram_addr.fine_y() as u16) + 8);
                    },
                    7 => {
                        self.increment_scroll_x();
                    },
                    _ => ()
                }
            }

            if self.cycle == 256 {
                self.increment_scroll_y();
            }

            if self.cycle == 257 {
                self.load_background_shifters();
                self.transfer_address_x();
            }

            if self.cycle == 338 || self.cycle == 340 {
                self.bg_next_tile_id = self.ppu_read(0x2000 | (self.vram_addr.reg & 0x0FFF));
            }

            if (self.scanline == -1) && (self.cycle >= 280 && self.cycle < 305) {
                self.transfer_address_y();
            }
        }

        if self.scanline >= 241 && self.scanline < 261 {
            if self.scanline == 241 && self.cycle == 1 {
                self.status.insert(Status::VERTICAL_BLANK);
                if self.control.contains(PPUCTRL::ENABLE_NMI) {
                    self.nmi = true;
                }
            }
        }

        let mut bg_pixel: u8 = 0x00;
        let mut bg_palette: u8 = 0x00;

        if self.mask.contains(Mask::RENDER_BACKGROUND) {
            let bit_mux: u16 = 0x8000 >> self.fine_x;

            let p0_pixel: u8 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
            let p1_pixel: u8 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;

            bg_pixel = (p1_pixel << 1) | p0_pixel;

            let bg_pal0 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
            let bg_pal1 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
            bg_palette = (bg_pal1 << 1) | bg_pal0;
        }

        let index = self.scanline as u32 * WIDTH + self.cycle as u32;
        let shade = self.get_color_from_palette_ram(bg_palette, bg_pixel);
        if self.scanline >= 0 && self.scanline < HEIGHT as i16 {
            if let Some(pixel) = frame.chunks_exact_mut(4).nth(index as usize) {
                pixel.copy_from_slice(&shade);
            }
        }
        // if let Some(pixel) = frame.chunks_exact_mut(4).nth(index as usize) {
        //     pixel.copy_from_slice(&shade);
        // }
        self.cycle += 1;
        if self.cycle >= 341 as i16 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 as i16 {
                self.scanline = -1;
                self.frame_complete = true;
            }
        }
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> [u8; 4] {
        let addr = 0x3F00 + (palette << 2) as u16 + pixel as u16;
        self.screen_palette[(self.ppu_read(addr) & 0x3F) as usize]
    }
}