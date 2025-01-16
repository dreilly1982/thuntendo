#![allow(unused)] // clear warning for chr_banks
use crate::mappers::mapper::Mapper;

pub struct Mapper000 {
    prg_banks: u8,
    chr_banks: u8
}

impl Mapper000 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Mapper000 {
        Mapper000 {
            prg_banks: prg_banks,
            chr_banks: chr_banks
        }
    }
}

impl Mapper for Mapper000{
    fn cpu_map_read(&self, addr: u16) -> (u32, bool) {
        match self.prg_banks{
            1 => (addr as u32 & 0x3FFF, true),
            _ => (addr as u32 & 0x7FFF, true)
        }
    }

    fn cpu_map_write(&mut self, addr: u16, _: u8) -> (u32, bool) {
        match self.prg_banks {
            1 => (addr as u32 & 0x3FFF, true),
            _ => (addr as u32 & 0x7FFF, true),
        }
    }

    fn ppu_map_read(&self, addr: u16) -> (u32, bool) {
        (addr as u32, true)
    }

    fn ppu_map_write(&mut self, addr: u16) -> (u32, bool) {
        (addr as u32, true)
    }

    fn reset(&mut self) {}
}