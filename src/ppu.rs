use crate::nes::NES;
use crate::cartridge::{Cartridge, MIRROR};
use crate::WIDTH;
use crate::HEIGHT;
use std::cell::RefCell;
use std::rc::Rc;
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

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct ObjectAttributeEntry {
    y: u8,
    id: u8,
    attribute: u8,
    x: u8,
}

impl ObjectAttributeEntry {
    fn as_byte(&self, index: u8) -> u8 {
        match index {
            0 => self.y,
            1 => self.id,
            2 => self.attribute,
            3 => self.x,
            _ => unreachable!("Invalid OAM index"),
        }
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
    pub oam: [ObjectAttributeEntry; 64],
    pub oam_addr: u8,
    sprite_scanline: [ObjectAttributeEntry; 8],
    sprite_count: u8,
    sprite_shifter_pattern_lo: [u8; 8],
    sprite_shifter_pattern_hi: [u8; 8],
    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,
    frame_buffer: [u8; (WIDTH * HEIGHT * 4) as usize],
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
            cycle: 0x0000,
            scanline: 0x0000,
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
            oam: [ObjectAttributeEntry { y: 0, id: 0, attribute: 0, x: 0 }; 64],
            oam_addr: 0x00,
            sprite_scanline: [ObjectAttributeEntry { y: 0, id: 0, attribute: 0, x: 0 }; 8],
            sprite_count: 0x00,
            sprite_shifter_pattern_lo: [0; 8],
            sprite_shifter_pattern_hi: [0; 8],
            sprite_zero_hit_possible: false,
            sprite_zero_being_rendered: false,
            frame_buffer: [0; (WIDTH * HEIGHT * 4) as usize],
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
            0x0004 => {
                data = self.oam[self.oam_addr as usize].as_byte(self.oam_addr % 4);
            },
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
            0x0003 => {
                self.oam_addr = data;
            },
            0x0004 => {
                self.oam_write(data);
            },
            0x0005 => {
                if self.address_latch == 0x00 {
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
                if self.address_latch == 0x00 {
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

    pub fn oam_write(&mut self, data: u8) {
        let entry_index = (self.oam_addr / 4) as usize;
        let byte_index = self.oam_addr % 4;
        match byte_index {
            0 => self.oam[entry_index].y = data,
            1 => self.oam[entry_index].id = data,
            2 => self.oam[entry_index].attribute = data,
            3 => self.oam[entry_index].x = data,
            _ => (),
        }
        self.oam_addr = self.oam_addr.wrapping_add(1);
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

        if self.mask.contains(Mask::RENDER_SPRITES) && self.cycle >= 1 && self.cycle < 258 {
            for i in 0..self.sprite_count {
                if self.sprite_scanline[i as usize].x > 0 {
                    self.sprite_scanline[i as usize].x -= 1;
                } else {
                    self.sprite_shifter_pattern_lo[i as usize] <<= 1;
                    self.sprite_shifter_pattern_hi[i as usize] <<= 1;
                }
            }
        }
    }

    pub fn clock(&mut self, frame: &mut [u8]) {

        if self.scanline >= -1 && self.scanline < 240 {
            if self.scanline == 0 && self.cycle == 0 {
                self.cycle = 1;
            }

            if self.scanline == -1 && self.cycle == 1 {
                self.status.remove(Status::VERTICAL_BLANK);
                self.status.remove(Status::SPRITE_ZERO_HIT);
                self.status.remove(Status::SPRITE_OVERFLOW);

                for i in 0..8 {
                    self.sprite_shifter_pattern_lo[i] = 0;
                    self.sprite_shifter_pattern_hi[i] = 0;
                }
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

            if self.scanline == -1 && self.cycle >= 280 && self.cycle < 305 {
                self.transfer_address_y();
            }

            if self.cycle == 257 && self.scanline >= 0 {
                self.sprite_scanline = [ObjectAttributeEntry { y: 0xFF, id: 0xFF, attribute: 0xFF, x: 0xFF }; 8];
                self.sprite_count = 0;

                for i in 0..8usize {
                    self.sprite_shifter_pattern_lo[i] = 0;
                    self.sprite_shifter_pattern_hi[i] = 0;
                }

                let mut oam_entry: u8 = 0;

                self.sprite_zero_hit_possible = false;

                while oam_entry < 64 && self.sprite_count < 9 {
                    let diff = self.scanline as i16 - self.oam[oam_entry as usize].y as i16;
                    if diff >= 0 && diff < if self.control.contains(PPUCTRL::SPRITE_SIZE) { 16 } else { 8 } {
                        if self.sprite_count < 8 {

                            if oam_entry == 0 {
                                self.sprite_zero_hit_possible = true;
                            }

                            self.sprite_scanline[self.sprite_count as usize] = self.oam[oam_entry as usize];
                            self.sprite_count += 1;
                        }
                    }
                    oam_entry += 1;
                }

                if self.sprite_count > 8 {
                    self.status.insert(Status::SPRITE_OVERFLOW);
                } else {
                    self.status.remove(Status::SPRITE_OVERFLOW);
                }
            }

            if self.cycle == 340 {
                for i in 0..8u8 {
                    let mut sprite_pattern_bits_lo: u8;
                    let mut sprite_pattern_bits_hi: u8;
                    let sprite_pattern_addr_lo: u16;
                    let sprite_pattern_addr_hi: u16;
    
                    if !self.control.contains(PPUCTRL::SPRITE_SIZE) {
                        if self.sprite_scanline[i as usize].attribute & 0x80 == 0 {
                            sprite_pattern_addr_lo = ((self.control.contains(PPUCTRL::PATTERN_SPRITE) as u16) << 12) 
                                                    | ((self.sprite_scanline[i as usize].id as u16) << 4) 
                                                    | (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16);
                        } else {
                            sprite_pattern_addr_lo = ((self.control.contains(PPUCTRL::PATTERN_SPRITE) as u16) << 12) 
                                                    | ((self.sprite_scanline[i as usize].id as u16) << 4)
                                                    | (7 - (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16));
                        }
                    } else {
                        if self.sprite_scanline[i as usize].attribute & 0x80 == 0 {
                            if (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16) < 8 {
                                sprite_pattern_addr_lo = (((self.sprite_scanline[i as usize].id as u16) & 0x01) << 12)
                                                        | (((self.sprite_scanline[i as usize].id as u16) & 0xFE) << 4)
                                                        | ((self.scanline as u16 - self.sprite_scanline[i as usize].y as u16) & 0x07);
                            } else {
                                sprite_pattern_addr_lo = (((self.sprite_scanline[i as usize].id as u16) & 0x01) << 12)
                                                        | ((((self.sprite_scanline[i as usize].id as u16) & 0xFE) + 1) << 4)
                                                        | ((self.scanline as u16 - self.sprite_scanline[i as usize].y as u16) & 0x07);
                            }
                        } else {
                            if (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16) < 8 {
                                sprite_pattern_addr_lo = (((self.sprite_scanline[i as usize].id as u16) & 0x01) << 12)
                                                        | (((self.sprite_scanline[i as usize].id as u16) & 0xFE) << 4)
                                                        | ((7 - (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16) & 0x07));
                            } else {
                                sprite_pattern_addr_lo = (((self.sprite_scanline[i as usize].id as u16) & 0x01) << 12)
                                                        | ((((self.sprite_scanline[i as usize].id as u16) & 0xFE) + 1) << 4)
                                                        | ((7 - (self.scanline as u16 - self.sprite_scanline[i as usize].y as u16)) & 0x07);
                            }
                        }
                    }

                    sprite_pattern_addr_hi = sprite_pattern_addr_lo + 8;

                    sprite_pattern_bits_lo = self.ppu_read(sprite_pattern_addr_lo);
                    sprite_pattern_bits_hi = self.ppu_read(sprite_pattern_addr_hi);

                    if self.sprite_scanline[i as usize].attribute & 0x40 != 0 {
                        let flipbyte = |mut b: u8| {
                            b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
                            b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
                            b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
                            b
                        };

                        sprite_pattern_bits_lo = flipbyte(sprite_pattern_bits_lo);
                        sprite_pattern_bits_hi = flipbyte(sprite_pattern_bits_hi);
                    }

                    self.sprite_shifter_pattern_lo[i as usize] = sprite_pattern_bits_lo;
                    self.sprite_shifter_pattern_hi[i as usize] = sprite_pattern_bits_hi;
                }
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

        let mut fg_pixel: u8 = 0x00;
        let mut fg_palette: u8 = 0x00;
        let mut fg_priority: u8 = 0x00;

        if self.mask.contains(Mask::RENDER_SPRITES) {
            self.sprite_zero_being_rendered = false;

            for i in 0..self.sprite_count {
                if self.sprite_scanline[i as usize].x == 0 {
                    let fg_pixel_lo: u8 = ((self.sprite_shifter_pattern_lo[i as usize] & 0x80) > 0) as u8;
                    let fg_pixel_hi: u8 = ((self.sprite_shifter_pattern_hi[i as usize] & 0x80) > 0) as u8;
                    fg_pixel = (fg_pixel_hi << 1) | fg_pixel_lo;

                    fg_palette = (self.sprite_scanline[i as usize].attribute & 0x03) + 0x04;
                    fg_priority = ((self.sprite_scanline[i as usize].attribute & 0x20) == 0) as u8;

                    if fg_pixel != 0 {

                        if i == 0 {
                            self.sprite_zero_being_rendered = true;
                        }

                        break;
                    }
                }
            }
        }

        let mut pixel: u8 = 0x00;
        let mut palette: u8 = 0x00;

        if bg_pixel == 0 && fg_pixel > 0 {
            pixel = fg_pixel;
            palette = fg_palette;
        } else if bg_pixel > 0 && fg_pixel == 0 {
            pixel = bg_pixel;
            palette = bg_palette;
        } else if bg_pixel > 0 && fg_pixel > 0 {
            if fg_priority > 0 {
                pixel = fg_pixel;
                palette = fg_palette;
            } else {
                pixel = bg_pixel;
                palette = bg_palette;
            }
        }

        if self.sprite_zero_hit_possible && self.sprite_zero_being_rendered {
            if self.mask.contains(Mask::RENDER_BACKGROUND) && self.mask.contains(Mask::RENDER_SPRITES) {
                if !(self.mask.contains(Mask::RENDER_BACKGROUND_LEFT) || self.mask.contains(Mask::RENDER_SPRITES_LEFT)) {
                    if self.cycle >= 9 && self.cycle < 258 {
                        self.status.insert(Status::SPRITE_ZERO_HIT);
                    }
                } else {
                    if self.cycle >= 1 && self.cycle < 258 {
                        self.status.insert(Status::SPRITE_ZERO_HIT);
                    }
                }
            }
        }

        let index = self.scanline as u32 * WIDTH + self.cycle as u32;
        let shade = self.get_color_from_palette_ram(palette, pixel);
        if self.scanline >= 0 && self.scanline < HEIGHT as i16 {
            if let Some(pixel) = frame.chunks_exact_mut(4).nth(index as usize) {
                pixel.copy_from_slice(&shade);
            }
        }

        self.cycle += 1;
        if self.cycle >= 341 as i16 {
            self.cycle = 0;
            self.scanline += 1;
            if self.scanline >= 261 as i16 {
                self.scanline = -1;
                self.frame_complete = true;
                // frame.copy_from_slice(&self.frame_buffer);
            }
        }
    }

    pub fn get_color_from_palette_ram(&self, palette: u8, pixel: u8) -> [u8; 4] {
        let addr = 0x3F00 + (palette << 2) as u16 + pixel as u16;
        self.screen_palette[(self.ppu_read(addr) & 0x3F) as usize]
    }
}