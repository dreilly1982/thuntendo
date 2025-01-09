use std::io::Error;
use std::{fs::File, io};
use std::io::prelude::*;

use crate::mappers::mapper::Mapper;
use crate::mappers::mapper_000::Mapper000;

#[derive(PartialEq)]
pub enum MIRROR {
    HORIZONTAL,
    VERTICAL,
    ONESCREEN_LO,
    ONESCREEN_HI,
}
pub struct Cartridge {
    num_prg_chunks: u8,
    num_chr_chunks: u8,
    mapper: Box<dyn Mapper>,
    pub mirroring: MIRROR,
    has_battery: bool,
    has_trainer: bool,
    has_four_screen_vram: bool,
    is_vs_unisystem: bool,
    is_playchoice10: bool,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
}

impl Cartridge {
    pub fn new(filename: &str) -> Cartridge {
        let mut f = File::open(filename).unwrap();

        let mut header: [u8; 16] = [0; 16];
        f.read_exact(&mut header).unwrap();
        assert!(header[0] == 0x4E);
        assert!(header[1] == 0x45);
        assert!(header[2] == 0x53);
        assert!(header[3] == 0x1A);
        let num_prg_chunks = header[4];
        let num_chr_chunks = header[5];
        let mut prg_rom: Vec<u8> = Vec::new();
        for _i in 0..num_prg_chunks {
            let mut bf: Vec<u8> = vec![0; 16384];
            f.read_exact(&mut bf).unwrap();
            prg_rom.append(&mut bf);
        }
        let mut chr_rom: Vec<u8> = Vec::new();
        for _i in 0..num_chr_chunks {
            let mut bf: Vec<u8> = vec![0; 8192];
            f.read_exact(&mut bf).unwrap();
            chr_rom.append(&mut bf);
        }
        let mapper_num = (header[6] >> 4) + ((header[7] >> 4) << 4);
        let mapper: Box<dyn Mapper> = {
            match mapper_num {
                0 => Box::new(Mapper000::new(num_prg_chunks, num_chr_chunks)),
                _ => panic!("INVALID MAPPER")
            }
        };
        let mirror = header[6] & 1;
        let mirroring = match mirror {
            0 => MIRROR::HORIZONTAL,
            1 => MIRROR::VERTICAL,
            _ => panic!("INVALID MIRRORING")
        };
        Cartridge {
            num_prg_chunks: num_prg_chunks,
            num_chr_chunks: num_chr_chunks,
            mirroring: mirroring,
            has_battery: ((header[6] >> 1) & 1) > 0,
            has_trainer: ((header[6] >> 2) & 1) > 0,
            has_four_screen_vram: ((header[6] >> 3) & 1) > 0,
            is_playchoice10: false,
            is_vs_unisystem: false,
            mapper: mapper,
            prg_rom: prg_rom,
            chr_rom: chr_rom,
        }
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        let mapped_addr = self.mapper.cpu_map_read(addr);
        self.prg_rom[mapped_addr as usize]
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        let mapped_addr = self.mapper.cpu_map_write(addr);
        self.prg_rom[mapped_addr as usize] = data;
    }

    pub fn ppu_read(&self, addr: u16) -> Result<u8, &'static str> {
        let mapped_addr = self.mapper.ppu_map_read(addr);
        self.chr_rom.get(mapped_addr as usize).copied().ok_or("PPU read error")
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) -> Result<(), &'static str> {
        let mapped_addr = self.mapper.ppu_map_write(addr);
        self.chr_rom.get_mut(mapped_addr as usize).map(|element| *element = data).ok_or("PPU write error")
    }
}
