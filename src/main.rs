mod bus;
mod cpu;
use bus::Bus;
use cpu::CPU;

fn main() {
    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    cpu.clock();
}
