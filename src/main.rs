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
    let mut buffer = [0; 24592];
    let f = File::open("/Users/edreill/repo/nes-test-roms/other/nestest.nes")?;
    let mut handle = f.take(24592);
    handle.read(&mut buffer)?;

    // let f = File::open("/Users/edreill/test.txt")?;

    let mut n_offset: u16 = 0xC000;

    // for byte in f.bytes() {
    //     bus.write(n_offset, byte.unwrap());
    //     n_offset += 1;
    // }

    for i in 16..(16 * 1024) {
        bus.write(n_offset, buffer[i]);
        n_offset += 1;
        // println!("{:02X}", buffer[i]);
    }

    bus.write(0xFFFC, 0x00);
    bus.write(0xFFFD, 0xC0);

    cpu.reset();

    while !cpu.complete() {
        cpu.clock();
    }
    Ok(())
}
