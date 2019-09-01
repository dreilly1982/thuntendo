mod bus;
mod cpu;
use bus::Bus;
use cpu::CPU;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

fn load_rom(bus: &Bus, fname: &str) {
    let metadata = fs::metadata(fname).unwrap();
    let rom_size = metadata.len();
    let mut buffer = vec![0; rom_size as usize];
    let f = File::open(fname).unwrap();
    let mut handle = f.take(rom_size);
    handle.read(&mut buffer).unwrap();

    if !(buffer[0] == 0x4E && buffer[1] == 0x45 && buffer[2] == 0x53 && buffer[3] == 0x1A) {
        println!("Invalid Filetype!");
        process::exit(1);
    }

    let prg_size: usize = buffer[4] as usize * 16 * 1024;
    let mut n_offset: u16 = 0x8000;

    if buffer[4] < 2 {
        for i in 16..16384 {
            bus.write(n_offset, buffer[i]);
            bus.write(n_offset + 0x4000, buffer[i]);
            n_offset += 1;
        }
    } else {
        for i in 16..32768 {
            bus.write(n_offset, buffer[i]);
            n_offset += 1;
        }
    }

    let chr_size: usize = buffer[5] as usize * 8 * 1024;

    bus.write(0xFFFC, 0x00);
    bus.write(0xFFFD, 0x80);
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let fname = &args[1];
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    load_rom(&bus, fname);

    cpu.reset();

    while !cpu.complete() {
        cpu.clock();
    }
    Ok(())
}

struct HiddenBytes(Vec<u8>);

pub struct Ines {
    num_prg_chunks: u8,
    num_chr_chunks: u8,
    mapper: u8,
    mirroring: bool,
    has_battery: bool,
    has_trainer: bool,
    has_four_screen_vram: bool,
    is_vs_unisystem: bool,
    is_playchoice10: bool,
    prg_rom: HiddenBytes,
    chr_rom: HiddenBytes,
}

pub fn read_ines(filename: &str) -> Result<Ines, io::Error> {
    let mut f = File::open(filename)?;

    let mut header: [u8; 16] = [0; 16];
    f.read_exact(&mut header)?;
    assert!(header[0] == 0x4E);
    assert!(header[1] == 0x45);
    assert!(header[2] == 0x53);
    assert!(header[3] == 0x1A);
    let num_prg_chunks = header[4];
    let num_chr_chunks = header[5];
    let mut prg_rom: Vec<u8> = Vec::new();
    for _i in 0..num_prg_chunks {
        let mut bf: Vec<u8> = vec![0; 16384];
        f.read_exact(&mut bf)?;
        prg_rom.append(&mut bf);
    }
    let mut chr_rom: Vec<u8> = Vec::new();
    for _i in 0..num_chr_chunks {
        let mut bf: Vec<u8> = vec![0; 8192];
        f.read_exact(&mut bf)?;
        chr_rom.append(&mut bf);
    }
    let ret = Ines {
        num_prg_chunks: num_prg_chunks,
        num_chr_chunks: num_chr_chunks,
        mirroring: ((header[6] >> 0) & 1) > 0,
        has_battery: ((header[6] >> 1) & 1) > 0,
        has_trainer: ((header[6] >> 2) & 1) > 0,
        has_four_screen_vram: ((header[6] >> 3) & 1) > 0,
        is_playchoice10: false,
        is_vs_unisystem: false,
        mapper: (header[6] >> 4) + ((header[7] >> 4) << 4),
        prg_rom: HiddenBytes(prg_rom),
        chr_rom: HiddenBytes(chr_rom),
    };
    return Ok(ret);
}
