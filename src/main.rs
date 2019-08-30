mod bus;
mod cpu;
use bus::Bus;
use cpu::CPU;
use std::fs::File;
use std::io;
use std::io::prelude::*;

fn main() -> io::Result<()> {
    let bus = Bus::new();
    let mut cpu = CPU::new(&bus);
    let f = File::open("/Users/edreill/test.txt")?;

    let mut n_offset: u16 = 0x8000;

    for byte in f.bytes() {
        bus.write(n_offset, byte.unwrap());
        n_offset += 1;
    }

    bus.write(0xFFFC, 0x00);
    bus.write(0xFFFD, 0x80);

    cpu.reset();

    while !cpu.complete() {
        cpu.clock();
    }
    Ok(())
}
