mod bus;
mod cpu;
use bus::Bus;
use cpu::CPU;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let f = File::open("/Users/edreill/test.txt")?;

    let mut n_offset: usize = 0x8000;

    for byte in f.bytes() {
        cpu.bus.ram[n_offset] = byte.unwrap();
        n_offset += 1;
    }

    cpu.bus.ram[0xFFFC] = 0x00;
    cpu.bus.ram[0xFFFD] = 0x80;

    cpu.reset();

    while !cpu.complete() {
        cpu.clock();
    }
    Ok(())
}
