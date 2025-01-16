#![allow(unused)] // clear warning for chr_banks
use crate::mappers::mapper::Mapper;

pub struct Mapper002 {
    prg_banks: u8,
    chr_banks: u8,
    prg_banks_select_lo: u8,
    prg_banks_select_hi: u8,
}

impl Mapper002 {
    pub fn new(prg_banks: u8, chr_banks: u8) -> Mapper002 {
        Mapper002 {
            prg_banks: prg_banks,
            chr_banks: chr_banks,
            prg_banks_select_lo: 0,
            prg_banks_select_hi: 0,
        }
    }
}

impl Mapper for Mapper002{
    fn cpu_map_read(&self, addr: u16) -> (u32, bool) {
        let mut ret_data = (0u32, false);
        if (addr >= 0x8000 && addr <= 0xBFFF) {
            ret_data = ((self.prg_banks_select_lo as u16 * 0x4000 + (addr & 0x3FFF)) as u32, true);
        }

        if (addr >= 0xC000) {
            ret_data = ((self.prg_banks_select_hi as u16 * 0x4000 + (addr & 0x3FFF)) as u32, true);
        }
        // println!("Mapper002 cpu_map_read: {:04X} {:02X} {}", addr, ret_data.0, ret_data.1);
        ret_data
    }

    fn cpu_map_write(&mut self, addr: u16, data: u8) -> (u32, bool) {
        if addr >= 0x8000 {
            self.prg_banks_select_lo = data & 0x0F;
        }
        (0, false)
    }

    fn ppu_map_read(&self, addr: u16) -> (u32, bool) {
        (addr as u32, true)
    }

    fn ppu_map_write(&mut self, addr: u16) -> (u32, bool) {
        (addr as u32, true)
    }

    fn reset(&mut self) {
        self.prg_banks_select_lo = 0;
        self.prg_banks_select_hi = self.prg_banks - 1;
    }
}