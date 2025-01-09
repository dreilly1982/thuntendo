#![allow(non_snake_case)]
use crate::nes::NES;
use std::rc::Rc;

pub enum FLAGS {
    C = (1 << 0),
    Z = (1 << 1),
    I = (1 << 2),
    D = (1 << 3),
    B = (1 << 4),
    U = (1 << 5),
    V = (1 << 6),
    N = (1 << 7),
}

pub struct CPU<'a> {
    pub bus: Rc<NES<'a>>,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub stkp: u8,
    pub pc: u16,
    pub status: u8,
    fetched: u8,
    temp: u16,
    addr_abs: u16,
    addr_rel: u16,
    opcode: u8,
    cycles: u8,
    clk_count: u16,
    lookup: [(&'static str, fn(&mut CPU) -> u8, fn(&mut CPU) -> u8, u8); 256],
}

impl<'a> CPU<'a> {
    pub fn new(b: Rc<NES<'a>>) -> CPU<'a> {
        CPU {
            bus: b,
            a: 0x00,
            x: 0x00,
            y: 0x00,
            stkp: 0x00,
            pc: 0x0000,
            status: 0x00,
            fetched: 0x00,
            temp: 0x0000,
            addr_abs: 0x0000,
            addr_rel: 0x000,
            opcode: 0x00,
            cycles: 0,
            clk_count: 0,
            lookup: [
                ("BRK", CPU::BRK, CPU::IMM, 7),
                ("ORA", CPU::ORA, CPU::IZX, 6),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("ORA", CPU::ORA, CPU::ZP0, 3),
                ("ASL", CPU::ASL, CPU::ZP0, 5),
                ("SLO", CPU::SLO, CPU::ZP0, 5),
                ("PHP", CPU::PHP, CPU::IMP, 3),
                ("ORA", CPU::ORA, CPU::IMM, 2),
                ("ASL", CPU::ASL, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("NOP", CPU::NOP, CPU::ABS, 4),
                ("ORA", CPU::ORA, CPU::ABS, 4),
                ("ASL", CPU::ASL, CPU::ABS, 6),
                ("SLO", CPU::SLO, CPU::ABS, 6),
                ("BPL", CPU::BPL, CPU::REL, 2),
                ("ORA", CPU::ORA, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("ORA", CPU::ORA, CPU::ZPX, 4),
                ("ASL", CPU::ASL, CPU::ZPX, 6),
                ("SLO", CPU::SLO, CPU::ZPX, 6),
                ("CLC", CPU::CLC, CPU::IMP, 2),
                ("ORA", CPU::ORA, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("ORA", CPU::ORA, CPU::ABX, 4),
                ("ASL", CPU::ASL, CPU::ABX, 7),
                ("SLO", CPU::SLO, CPU::ABX, 7),
                ("JSR", CPU::JSR, CPU::ABS, 6),
                ("AND", CPU::AND, CPU::IZX, 6),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::IZX, 8),
                ("BIT", CPU::BIT, CPU::ZP0, 3),
                ("AND", CPU::AND, CPU::ZP0, 3),
                ("ROL", CPU::ROL, CPU::ZP0, 5),
                ("RLA", CPU::RLA, CPU::ZP0, 5),
                ("PLP", CPU::PLP, CPU::IMP, 4),
                ("AND", CPU::AND, CPU::IMM, 2),
                ("ROL", CPU::ROL, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("BIT", CPU::BIT, CPU::ABS, 4),
                ("AND", CPU::AND, CPU::ABS, 4),
                ("ROL", CPU::ROL, CPU::ABS, 6),
                ("RLA", CPU::RLA, CPU::ABS, 6),
                ("BMI", CPU::BMI, CPU::REL, 2),
                ("AND", CPU::AND, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("AND", CPU::AND, CPU::ZPX, 4),
                ("ROL", CPU::ROL, CPU::ZPX, 6),
                ("RLA", CPU::RLA, CPU::ZPX, 6),
                ("SEC", CPU::SEC, CPU::IMP, 2),
                ("AND", CPU::AND, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("AND", CPU::AND, CPU::ABX, 4),
                ("ROL", CPU::ROL, CPU::ABX, 7),
                ("RLA", CPU::RLA, CPU::ABX, 7),
                ("RTI", CPU::RTI, CPU::IMP, 6),
                ("EOR", CPU::EOR, CPU::IZX, 6),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("EOR", CPU::EOR, CPU::ZP0, 3),
                ("LSR", CPU::LSR, CPU::ZP0, 5),
                ("SRE", CPU::SRE, CPU::ZP0, 5),
                ("PHA", CPU::PHA, CPU::IMP, 3),
                ("EOR", CPU::EOR, CPU::IMM, 2),
                ("LSR", CPU::LSR, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("JMP", CPU::JMP, CPU::ABS, 3),
                ("EOR", CPU::EOR, CPU::ABS, 4),
                ("LSR", CPU::LSR, CPU::ABS, 6),
                ("SRE", CPU::SRE, CPU::ABS, 6),
                ("BVC", CPU::BVC, CPU::REL, 2),
                ("EOR", CPU::EOR, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("EOR", CPU::EOR, CPU::ZPX, 4),
                ("LSR", CPU::LSR, CPU::ZPX, 6),
                ("SRE", CPU::SRE, CPU::ZPX, 6),
                ("CLI", CPU::CLI, CPU::IMP, 2),
                ("EOR", CPU::EOR, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("EOR", CPU::EOR, CPU::ABX, 4),
                ("LSR", CPU::LSR, CPU::ABX, 7),
                ("SRE", CPU::SRE, CPU::ABX, 7),
                ("RTS", CPU::RTS, CPU::IMP, 6),
                ("ADC", CPU::ADC, CPU::IZX, 6),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("RRA", CPU::RRA, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("ADC", CPU::ADC, CPU::ZP0, 3),
                ("ROR", CPU::ROR, CPU::ZP0, 5),
                ("RRA", CPU::RRA, CPU::ZP0, 5),
                ("PLA", CPU::PLA, CPU::IMP, 4),
                ("ADC", CPU::ADC, CPU::IMM, 2),
                ("ROR", CPU::ROR, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("JMP", CPU::JMP, CPU::IND, 5),
                ("ADC", CPU::ADC, CPU::ABS, 4),
                ("ROR", CPU::ROR, CPU::ABS, 6),
                ("RRA", CPU::RRA, CPU::ABS, 6),
                ("BVS", CPU::BVS, CPU::REL, 2),
                ("ADC", CPU::ADC, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("RRA", CPU::RRA, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("ADC", CPU::ADC, CPU::ZPX, 4),
                ("ROR", CPU::ROR, CPU::ZPX, 6),
                ("RRA", CPU::RRA, CPU::ZPX, 6),
                ("SEI", CPU::SEI, CPU::IMP, 2),
                ("ADC", CPU::ADC, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("RRA", CPU::RRA, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("ADC", CPU::ADC, CPU::ABX, 4),
                ("ROR", CPU::ROR, CPU::ABX, 7),
                ("RRA", CPU::RRA, CPU::ABX, 7),
                ("NOP", CPU::NOP, CPU::IMM, 2),
                ("STA", CPU::STA, CPU::IZX, 6),
                ("NOP", CPU::NOP, CPU::IMM, 2),
                ("SAX", CPU::SAX, CPU::IZX, 6),
                ("STY", CPU::STY, CPU::ZP0, 3),
                ("STA", CPU::STA, CPU::ZP0, 3),
                ("STX", CPU::STX, CPU::ZP0, 3),
                ("SAX", CPU::SAX, CPU::ZP0, 3),
                ("DEY", CPU::DEY, CPU::IMP, 2),
                ("NOP", CPU::NOP, CPU::IMM, 2),
                ("TXA", CPU::TXA, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("STY", CPU::STY, CPU::ABS, 4),
                ("STA", CPU::STA, CPU::ABS, 4),
                ("STX", CPU::STX, CPU::ABS, 4),
                ("SAX", CPU::SAX, CPU::ABS, 4),
                ("BCC", CPU::BCC, CPU::REL, 2),
                ("STA", CPU::STA, CPU::IZY, 6),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 6),
                ("STY", CPU::STY, CPU::ZPX, 4),
                ("STA", CPU::STA, CPU::ZPX, 4),
                ("STX", CPU::STX, CPU::ZPY, 4),
                ("SAX", CPU::SAX, CPU::ZPY, 4),
                ("TYA", CPU::TYA, CPU::IMP, 2),
                ("STA", CPU::STA, CPU::ABY, 5),
                ("TXS", CPU::TXS, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 5),
                ("NOP", CPU::NOP, CPU::IMP, 5),
                ("STA", CPU::STA, CPU::ABX, 5),
                ("NOP", CPU::XXX, CPU::IMP, 5),
                ("NOP", CPU::XXX, CPU::IMP, 5),
                ("LDY", CPU::LDY, CPU::IMM, 2),
                ("LDA", CPU::LDA, CPU::IZX, 6),
                ("LDX", CPU::LDX, CPU::IMM, 2),
                ("LAX", CPU::LAX, CPU::IZX, 6),
                ("LDY", CPU::LDY, CPU::ZP0, 3),
                ("LDA", CPU::LDA, CPU::ZP0, 3),
                ("LDX", CPU::LDX, CPU::ZP0, 3),
                ("LAX", CPU::LAX, CPU::ZP0, 3),
                ("TAY", CPU::TAY, CPU::IMP, 2),
                ("LDA", CPU::LDA, CPU::IMM, 2),
                ("TAX", CPU::TAX, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("LDY", CPU::LDY, CPU::ABS, 4),
                ("LDA", CPU::LDA, CPU::ABS, 4),
                ("LDX", CPU::LDX, CPU::ABS, 4),
                ("LAX", CPU::LAX, CPU::ABS, 4),
                ("BCS", CPU::BCS, CPU::REL, 2),
                ("LDA", CPU::LDA, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("LAX", CPU::LAX, CPU::IZY, 5),
                ("LDY", CPU::LDY, CPU::ZPX, 4),
                ("LDA", CPU::LDA, CPU::ZPX, 4),
                ("LDX", CPU::LDX, CPU::ZPY, 4),
                ("LAX", CPU::LAX, CPU::ZPY, 4),
                ("CLV", CPU::CLV, CPU::IMP, 2),
                ("LDA", CPU::LDA, CPU::ABY, 4),
                ("TSX", CPU::TSX, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 4),
                ("LDY", CPU::LDY, CPU::ABX, 4),
                ("LDA", CPU::LDA, CPU::ABX, 4),
                ("LDX", CPU::LDX, CPU::ABY, 4),
                ("LAX", CPU::LAX, CPU::ABY, 4),
                ("CPY", CPU::CPY, CPU::IMM, 2),
                ("CMP", CPU::CMP, CPU::IZX, 6),
                ("NOP", CPU::NOP, CPU::IMM, 2),
                ("DCP", CPU::DCP, CPU::IZX, 8),
                ("CPY", CPU::CPY, CPU::ZP0, 3),
                ("CMP", CPU::CMP, CPU::ZP0, 3),
                ("DEC", CPU::DEC, CPU::ZP0, 5),
                ("DCP", CPU::DCP, CPU::ZP0, 5),
                ("INY", CPU::INY, CPU::IMP, 2),
                ("CMP", CPU::CMP, CPU::IMM, 2),
                ("DEX", CPU::DEX, CPU::IMP, 2),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("CPY", CPU::CPY, CPU::ABS, 4),
                ("CMP", CPU::CMP, CPU::ABS, 4),
                ("DEC", CPU::DEC, CPU::ABS, 6),
                ("DCP", CPU::DCP, CPU::ABS, 6),
                ("BNE", CPU::BNE, CPU::REL, 2),
                ("CMP", CPU::CMP, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("DCP", CPU::DCP, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("CMP", CPU::CMP, CPU::ZPX, 4),
                ("DEC", CPU::DEC, CPU::ZPX, 6),
                ("DCP", CPU::DCP, CPU::ZPX, 6),
                ("CLD", CPU::CLD, CPU::IMP, 2),
                ("CMP", CPU::CMP, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("DCP", CPU::DCP, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("CMP", CPU::CMP, CPU::ABX, 4),
                ("DEC", CPU::DEC, CPU::ABX, 7),
                ("DCP", CPU::DCP, CPU::ABX, 7),
                ("CPX", CPU::CPX, CPU::IMM, 2),
                ("SBC", CPU::SBC, CPU::IZX, 6),
                ("NOP", CPU::NOP, CPU::IMM, 2),
                ("ISB", CPU::ISB, CPU::IZX, 8),
                ("CPX", CPU::CPX, CPU::ZP0, 3),
                ("SBC", CPU::SBC, CPU::ZP0, 3),
                ("INC", CPU::INC, CPU::ZP0, 5),
                ("ISB", CPU::ISB, CPU::ZP0, 5),
                ("INX", CPU::INX, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::IMM, 2),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::IMM, 2),
                ("CPX", CPU::CPX, CPU::ABS, 4),
                ("SBC", CPU::SBC, CPU::ABS, 4),
                ("INC", CPU::INC, CPU::ABS, 6),
                ("ISB", CPU::ISB, CPU::ABS, 6),
                ("BEQ", CPU::BEQ, CPU::REL, 2),
                ("SBC", CPU::SBC, CPU::IZY, 5),
                ("NOP", CPU::XXX, CPU::IMP, 2),
                ("ISB", CPU::ISB, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("SBC", CPU::SBC, CPU::ZPX, 4),
                ("INC", CPU::INC, CPU::ZPX, 6),
                ("ISB", CPU::ISB, CPU::ZPX, 6),
                ("SED", CPU::SED, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("ISB", CPU::ISB, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("SBC", CPU::SBC, CPU::ABX, 4),
                ("INC", CPU::INC, CPU::ABX, 7),
                ("ISB", CPU::ISB, CPU::ABX, 7),
            ],
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.cpu_read(addr)
    }

    fn write(&self, addr: u16, data: u8) {
        self.bus.cpu_write(addr, data);
    }

    fn set_a(&mut self, val: u8) {
        self.a = val;
    }

    fn set_x(&mut self, val: u8) {
        self.x = val;
    }

    fn set_y(&mut self, val: u8) {
        self.y = val;
    }

    fn set_stkp(&mut self, val: u8) {
        self.stkp = val;
    }

    fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    fn set_status(&mut self, val: u8) {
        self.status = val;
    }

    fn set_fetched(&mut self, val: u8) {
        self.fetched = val;
    }

    fn set_temp(&mut self, val: u16) {
        self.temp = val;
    }

    fn set_addr_abs(&mut self, val: u16) {
        self.addr_abs = val;
    }

    fn set_addr_rel(&mut self, val: u16) {
        self.addr_rel = val;
    }

    fn set_opcode(&mut self, val: u8) {
        self.opcode = val;
    }

    fn set_cycles(&mut self, val: u8) {
        self.cycles = val;
    }

    fn set_clock_count(&mut self, val: u16) {
        self.clk_count = val;
    }

    // fn set_clock_count(&self, val: u32) {
    //     let mut lock = self.clock_count.write().unwrap();
    //     *lock = val;
    // }

    fn get_a(&self) -> u8 {
        self.a
    }

    fn get_x(&self) -> u8 {
        self.x
    }

    fn get_y(&self) -> u8 {
        self.y
    }

    fn get_stkp(&self) -> u8 {
        self.stkp
    }

    fn get_pc(&self) -> u16 {
       self.pc
    }

    fn get_status(&self) -> u8 {
        self.status
    }

    fn get_fetched(&self) -> u8 {
        self.fetched
    }

    fn get_temp(&self) -> u16 {
        self.temp
    }

    fn get_addr_abs(&self) -> u16 {
        self.addr_abs
    }

    fn get_addr_rel(&self) -> u16 {
        self.addr_rel
    }

    fn get_opcode(&self) -> u8 {
        self.opcode
    }

    fn get_cycles(&self) -> u8 {
        self.cycles 
    }

    fn get_clock_count(&self) -> u16 {
        self.clk_count
    }

    // fn get_clock_count(&self) -> u32 {
    //     return *self.clock_count.read().unwrap();
    // }

    pub fn reset(&mut self) {
        self.set_addr_abs(0xFFFC);
        let lo: u16 = self.read(self.get_addr_abs().wrapping_add(0)).into();
        let hi: u16 = self.read(self.get_addr_abs().wrapping_add(1)).into();

        self.set_pc((hi << 8) | lo);

        self.set_a(0);
        self.set_x(0);
        self.set_y(0);
        self.set_stkp(0xFD);
        self.set_status(0x00 | FLAGS::U as u8 | FLAGS::I as u8);

        self.set_addr_rel(0x0000);
        self.set_addr_abs(0x0000);
        self.set_fetched(0x00);

        self.set_clock_count(0);
        self.set_cycles(8);
    }

    pub fn irq(&mut self) {
        if self.get_flag(FLAGS::I) == 0 {
            self.write(
                (0x0100 as u16).wrapping_add(self.get_stkp() as u16),
                ((self.get_pc() >> 8) & 0x00FF) as u8,
            );
            self.set_stkp(self.get_stkp().wrapping_sub(1));
            self.write(
                (0x0100 as u16).wrapping_add(self.get_stkp() as u16),
                (self.get_pc() & 0x00FF) as u8,
            );

            self.set_flag(FLAGS::B, false);
            self.set_flag(FLAGS::U, true);
            self.set_flag(FLAGS::I, true);
            self.write(
                (0x100 as u16).wrapping_add(self.get_stkp() as u16),
                self.get_status(),
            );
            self.set_stkp(self.get_stkp().wrapping_sub(1));

            self.set_addr_abs(0xFFFE);
            let lo: u16 = self.read(self.get_addr_abs().wrapping_add(0)) as u16;
            let hi: u16 = self.read(self.get_addr_abs().wrapping_add(1)) as u16;
            self.set_pc((hi << 8) | lo);

            self.set_cycles(7);
        }
    }

    pub fn nmi(&mut self) {
        self.write(
            (0x0100 as u16).wrapping_add(self.get_stkp() as u16),
            ((self.get_pc() >> 8) & 0x00FF) as u8,
        );
        self.set_stkp(self.get_stkp().wrapping_sub(1));
        self.write(
            (0x0100 as u16).wrapping_add(self.get_stkp() as u16),
            (self.get_pc() & 0x00FF) as u8,
        );

        self.set_flag(FLAGS::B, false);
        self.set_flag(FLAGS::U, true);
        self.set_flag(FLAGS::I, true);
        self.write(
            (0x100 as u16).wrapping_add(self.get_stkp() as u16),
            self.get_status(),
        );
        self.set_stkp(self.get_stkp().wrapping_sub(1));

        self.set_addr_abs(0xFFFA);
        let lo: u16 = self.read(self.get_addr_abs().wrapping_add(0)) as u16;
        let hi: u16 = self.read(self.get_addr_abs().wrapping_add(1)) as u16;
        self.set_pc((hi << 8) | lo);
        self.set_cycles(8);
    }

    pub fn clock(&mut self) {
        self.set_cycles(self.get_cycles().wrapping_sub(1));
        self.set_clock_count(self.get_clock_count().wrapping_add(1));
        if self.get_cycles() == 0 {
            self.set_opcode(self.read(self.get_pc()));
            if self.get_pc() < 0x300 {
                return;
            }
            println!(
                "{:04X} {:02X} {}   A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                self.get_pc(),
                self.get_opcode(),
                self.lookup[self.get_opcode() as usize].0,
                self.get_a(),
                self.get_x(),
                self.get_y(),
                self.get_status(),
                self.get_stkp(),
                self.get_clock_count()
            );
            self.set_flag(FLAGS::U, true);
            let old_pc = self.get_pc();
            self.set_pc(self.get_pc().wrapping_add(1));
            // if self.get_pc() < 0x7000 {
            //     println!("Old PC: {:04X} New PC: {:04X}", old_pc, self.get_pc());
            // }
            self.set_cycles(self.lookup[self.get_opcode() as usize].3);
            let additional_cycle1: u8 = self.lookup[self.get_opcode() as usize].2(self);
            let additional_cycle2: u8 = self.lookup[self.get_opcode() as usize].1(self);
            self.set_status(self.get_status() & 0xEF);
            self.set_cycles(
                self.get_cycles()
                    .wrapping_add(additional_cycle1 & additional_cycle2),
            );
            self.set_flag(FLAGS::U, true);
        }
    }

    fn get_flag(&self, f: FLAGS) -> u8 {
        if self.get_status() & f as u8 > 0 {
            return 1;
        } else {
            return 0;
        }
    }

    fn set_flag(&mut self, f: FLAGS, v: bool) {
        if v {
            self.set_status(self.get_status() | (f as u8));
        } else {
            self.set_status(self.get_status() & !(f as u8));
        }
    }

    fn IMP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_fetched(x.get_a());
        return 0;
    }

    fn IMM<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_addr_abs(x.get_pc());
        x.set_pc(x.get_pc().wrapping_add(1));
        return 0;
    }

    fn ZP0<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_addr_abs(x.read(x.get_pc()) as u16);
        x.set_pc(x.get_pc().wrapping_add(1));
        x.set_addr_abs(x.get_addr_abs() & 0x00FF);
        return 0;
    }

    fn ZPX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_addr_abs((x.read(x.get_pc()).wrapping_add(x.get_x())) as u16);
        x.set_pc(x.get_pc().wrapping_add(1));
        x.set_addr_abs(x.get_addr_abs() & 0x00FF);
        return 0;
    }

    fn ZPY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_addr_abs((x.read(x.get_pc()).wrapping_add(x.get_y())) as u16);
        x.set_pc(x.get_pc().wrapping_add(1));
        return 0;
    }

    fn REL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_addr_rel(x.read(x.get_pc()) as u16);
        x.set_pc(x.get_pc().wrapping_add(1));
        if (x.get_addr_rel() & 0x80) != 0 {
            x.set_addr_rel(x.get_addr_rel() | 0xFF00);
        }
        return 0;
    }

    fn ABS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));
        let hi = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));

        x.set_addr_abs((hi << 8) | lo);

        return 0;
    }

    fn ABX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));
        let hi = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));

        x.set_addr_abs((hi << 8) | lo);
        x.set_addr_abs(x.get_addr_abs().wrapping_add(x.get_x() as u16));
        if (x.get_addr_abs() & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn ABY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));
        let hi = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));

        x.set_addr_abs((hi << 8) | lo);
        x.set_addr_abs(x.get_addr_abs().wrapping_add(x.get_y() as u16));

        if (x.get_addr_abs() & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn IND<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let ptr_lo = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));
        let ptr_hi = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));

        let ptr = (ptr_hi << 8) | ptr_lo;

        if ptr_lo == 0x00FF {
            x.set_addr_abs(
                ((x.read(ptr & 0xFF00) as u16) << 8) | x.read(ptr.wrapping_add(0)) as u16,
            );
        } else {
            x.set_addr_abs(
                (x.read(ptr.wrapping_add(1)) as u16) << 8 | x.read(ptr.wrapping_add(0)) as u16,
            );
        }
        return 0;
    }

    fn IZX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let t = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));
        let lo = x.read((t.wrapping_add(x.get_x() as u16)) & 0x00FF) as u16;
        let hi = x.read((t.wrapping_add(x.get_x() as u16)).wrapping_add(1) & 0x00FF) as u16;

        x.set_addr_abs((hi << 8) | lo);

        return 0;
    }

    fn IZY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let t = x.read(x.get_pc()) as u16;
        x.set_pc(x.get_pc().wrapping_add(1));

        let lo = x.read(t & 0x00FF) as u16;
        let hi = x.read((t.wrapping_add(1)) & 0x00FF) as u16;

        x.set_addr_abs((hi << 8) | lo);
        x.set_addr_abs(x.get_addr_abs().wrapping_add(x.get_y() as u16));

        if (x.get_addr_abs() & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn fetch(&mut self) {
        if self.lookup[self.get_opcode() as usize].2 as usize != CPU::IMP as usize {
            self.set_fetched(self.read(self.get_addr_abs()));
        }
        // return self.fetched;
    }

    fn ADC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();

        x.set_temp(
            (x.get_a() as u16)
                .wrapping_add(x.get_fetched() as u16)
                .wrapping_add(x.get_flag(FLAGS::C) as u16),
        );
        x.set_flag(FLAGS::C, x.get_temp() > 255);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0);
        x.set_flag(
            FLAGS::V,
            ((!(x.get_a() as u16 ^ x.get_fetched() as u16)
                & (x.get_a() as u16 ^ x.get_temp() as u16))
                & 0x0080)
                != 0,
        );
        x.set_flag(FLAGS::N, (x.get_temp() & 0x80) != 0);

        x.set_a((x.get_temp() & 0x00FF) as u8);
        return 1;
    }

    fn SBC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();

        let value = x.get_fetched() as u16 ^ 0x00FF;

        x.set_temp(
            (x.get_a() as u16)
                .wrapping_add(value)
                .wrapping_add(x.get_flag(FLAGS::C) as u16),
        );
        x.set_flag(FLAGS::C, (x.get_temp() & 0xFF00) != 0);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0);
        x.set_flag(
            FLAGS::V,
            ((x.get_temp() ^ x.get_a() as u16) & (x.get_temp() ^ value) & 0x0080) != 0,
        );
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        x.set_a((x.get_temp() & 0x00FF) as u8);
        return 1;
    }

    fn AND<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_a(x.get_a() & x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 1;
    }

    fn ASL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_fetched() as u16) << 1);
        x.set_flag(FLAGS::C, (x.get_temp() & 0xFF00) > 0);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x80) != 0);
        if x.lookup[x.get_opcode() as usize].2 as usize == CPU::IMP as usize {
            x.set_a((x.get_temp() & 0x00FF) as u8);
        } else {
            x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8)
        }
        return 0;
    }

    fn BCC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::C) == 0 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BCS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::C) == 1 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BEQ<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::Z) == 1 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BIT<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_a() & x.get_fetched()) as u16);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.get_fetched() & (1 << 7)) != 0);
        x.set_flag(FLAGS::V, (x.get_fetched() & (1 << 6)) != 0);
        return 0;
    }

    fn BMI<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::N) == 1 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BNE<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::Z) == 0 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BPL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::N) == 0 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BRK<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_pc(x.get_pc().wrapping_add(1));

        x.set_flag(FLAGS::I, true);
        x.write(
            (x.get_stkp() as u16).wrapping_add(0x0100),
            ((x.get_pc() >> 8) & 0x00ff) as u8,
        );
        x.set_stkp(x.get_stkp().wrapping_sub(1));
        x.write(
            (x.get_stkp() as u16).wrapping_add(0x0100),
            (x.get_pc() & 0x00FF) as u8,
        );
        x.set_stkp(x.get_stkp().wrapping_sub(1));

        x.set_flag(FLAGS::B, true);
        x.write((x.get_stkp() as u16).wrapping_add(0x0100), x.get_status());
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_flag(FLAGS::B, false);

        x.set_pc(x.read(0xFFFE) as u16 | ((x.read(0xFFFF) as u16) << 8));
        return 0;
    }

    fn BVC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::V) == 0 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn BVS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::V) == 1 {
            x.set_cycles(x.get_cycles().wrapping_add(1));
            x.set_addr_abs(x.get_pc().wrapping_add(x.get_addr_rel()));

            if (x.get_addr_abs() & 0xFF00) != (x.get_pc() & 0xFF00) {
                x.set_cycles(x.get_cycles().wrapping_add(1));
            }

            x.set_pc(x.get_addr_abs());
        }
        return 0;
    }

    fn CLC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::C, false);
        return 0;
    }

    fn CLD<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::D, false);
        return 0;
    }

    fn CLI<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::I, false);
        return 0;
    }

    fn CLV<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::V, false);
        return 0;
    }

    fn CMP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_a() as u16).wrapping_sub(x.get_fetched() as u16));
        x.set_flag(FLAGS::C, x.get_a() >= x.get_fetched());
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        return 1;
    }

    fn CPX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_x() as u16).wrapping_sub(x.get_fetched() as u16));
        x.set_flag(FLAGS::C, x.get_x() >= x.get_fetched());
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        return 0;
    }

    fn CPY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_y() as u16).wrapping_sub(x.get_fetched() as u16));
        x.set_flag(FLAGS::C, x.get_y() >= x.get_fetched());
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        return 0;
    }

    fn DCP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::DEC(x);
        CPU::CMP(x);
        return 0;
    }

    fn DEC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_fetched() as u16).wrapping_sub(1));
        x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        return 0;
    }

    fn DEX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_x(x.get_x().wrapping_sub(1));
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 0;
    }

    fn DEY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_y(x.get_y().wrapping_sub(1));
        x.set_flag(FLAGS::Z, x.get_y() == 0x00);
        x.set_flag(FLAGS::N, (x.get_y() & 0x80) != 0);
        return 0;
    }

    fn EOR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_a(x.get_a() ^ x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 1;
    }

    fn INC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_fetched().wrapping_add(1)) as u16);
        x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        return 0;
    }

    fn INX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_x(x.get_x().wrapping_add(1));
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 0;
    }

    fn INY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_y(x.get_y().wrapping_add(1));
        x.set_flag(FLAGS::Z, x.get_y() == 0x00);
        x.set_flag(FLAGS::N, (x.get_y() & 0x80) != 0);
        return 0;
    }

    fn ISB<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::INC(x);
        CPU::SBC(x);
        return 0;
    }

    fn JMP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_pc(x.get_addr_abs());
        return 0;
    }

    fn JSR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_pc(x.get_pc().wrapping_sub(1));
        x.write(
            (0x0100 as u16).wrapping_add(x.get_stkp() as u16),
            ((x.get_pc() >> 8) & 0x00FF) as u8,
        );
        x.set_stkp(x.get_stkp().wrapping_sub(1));
        x.write(
            (0x0100 as u16).wrapping_add(x.get_stkp() as u16),
            (x.get_pc() & 0x00FF) as u8,
        );
        x.set_stkp(x.get_stkp().wrapping_sub(1));

        x.set_pc(x.get_addr_abs());
        return 0;
    }

    fn LAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_a(x.get_fetched());
        x.set_x(x.get_a());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 0;
    }

    fn LDA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_a(x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 1;
    }

    fn LDX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_x(x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 1;
    }

    fn LDY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_y(x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_y() == 0x00);
        x.set_flag(FLAGS::N, (x.get_y() & 0x80) != 0);
        return 1;
    }

    fn LSR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_flag(FLAGS::C, (x.get_fetched() & 0x0001) != 0);
        x.set_temp((x.get_fetched() >> 1) as u16);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        if x.lookup[x.get_opcode() as usize].2 as usize == CPU::IMP as usize {
            x.set_a((x.get_temp() & 0x00FF) as u8);
        } else {
            x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8);
        }
        return 0;
    }

    fn NOP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let rc;
        match x.get_opcode() {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => rc = 1,
            _ => rc = 0,
        }
        return rc;
    }

    fn ORA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_a(x.get_a() | x.get_fetched());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 1;
    }

    fn PHA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write((x.get_stkp() as u16).wrapping_add(0x0100), x.get_a());
        x.set_stkp(x.get_stkp().wrapping_sub(1));
        return 0;
    }

    fn PHP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(
            (x.get_stkp() as u16).wrapping_add(0x0100),
            x.get_status() | FLAGS::B as u8 | FLAGS::U as u8,
        );
        x.set_flag(FLAGS::B, false);
        x.set_flag(FLAGS::U, false);
        x.set_stkp(x.get_stkp().wrapping_sub(1));
        return 0;
    }

    fn PLA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_a(x.read((x.get_stkp() as u16).wrapping_add(0x0100)));
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 0;
    }

    fn PLP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_status(x.read((x.get_stkp() as u16).wrapping_add(0x0100)));
        x.set_flag(FLAGS::U, true);
        return 0;
    }

    fn ROL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp(((x.get_fetched() as u16) << 1) | x.get_flag(FLAGS::C) as u16);
        x.set_flag(FLAGS::C, (x.get_temp() & 0xFF00) != 0);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        if x.lookup[x.get_opcode() as usize].2 as usize == CPU::IMP as usize {
            x.set_a((x.get_temp() & 0x00FF) as u8);
        } else {
            x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8);
        }
        return 0;
    }

    fn ROR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_temp((x.get_flag(FLAGS::C) << 7) as u16 | (x.get_fetched() >> 1) as u16);
        x.set_flag(FLAGS::C, (x.get_fetched() & 0x01) != 0);
        x.set_flag(FLAGS::Z, (x.get_temp() & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.get_temp() & 0x0080) != 0);
        if x.lookup[x.get_opcode() as usize].2 as usize == CPU::IMP as usize {
            x.set_a((x.get_temp() & 0x00FF) as u8);
        } else {
            x.write(x.get_addr_abs(), (x.get_temp() & 0x00FF) as u8);
        }
        return 0;
    }

    fn RLA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::ROL(x);
        CPU::AND(x);
        return 0;
    }

    fn RRA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::ROR(x);
        CPU::ADC(x);
        return 0;
    }

    fn RTI<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_status(x.read((x.get_stkp() as u16).wrapping_add(0x0100)));
        x.set_status(x.get_status() & !(FLAGS::B as u8));
        x.set_status(x.get_status() & !(FLAGS::U as u8));

        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_pc(x.read((x.get_stkp() as u16).wrapping_add(0x0100)) as u16);
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_pc(x.get_pc() | (x.read((x.get_stkp() as u16).wrapping_add(0x0100)) as u16) << 8);
        return 0;
    }

    fn RTS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_pc(x.read((x.get_stkp() as u16).wrapping_add(0x0100)) as u16);
        x.set_stkp(x.get_stkp().wrapping_add(1));
        x.set_pc(x.get_pc() | (x.read((x.get_stkp() as u16).wrapping_add(0x0100)) as u16) << 8);

        x.set_pc(x.get_pc().wrapping_add(1));
        return 0;
    }

    fn SAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.get_addr_abs(), x.get_a() & x.get_x());
        return 0;
    }

    fn SEC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::C, true);
        return 0;
    }

    fn SED<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::D, true);
        return 0;
    }

    fn SEI<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_flag(FLAGS::I, true);
        return 0;
    }

    fn SLO<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::ASL(x);
        CPU::ORA(x);

        return 0;
    }

    fn SRE<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::LSR(x);
        CPU::EOR(x);
        return 0;
    }

    fn STA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.get_addr_abs(), x.get_a());
        return 0;
    }

    fn STX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.get_addr_abs(), x.get_x());
        return 0;
    }

    fn STY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.get_addr_abs(), x.get_y());
        return 0;
    }

    fn TAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_x(x.get_a());
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 0;
    }

    fn TAY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_y(x.get_a());
        x.set_flag(FLAGS::Z, x.get_y() == 0x00);
        x.set_flag(FLAGS::N, (x.get_y() & 0x80) != 0);
        return 0;
    }

    fn TSX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_x(x.get_stkp());
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 0;
    }

    fn TXA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_a(x.get_x());
        x.set_flag(FLAGS::Z, x.get_x() == 0x00);
        x.set_flag(FLAGS::N, (x.get_x() & 0x80) != 0);
        return 0;
    }

    fn TXS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_stkp(x.get_x());
        return 0;
    }

    fn TYA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.set_a(x.get_y());
        x.set_flag(FLAGS::Z, x.get_a() == 0x00);
        x.set_flag(FLAGS::N, (x.get_a() & 0x80) != 0);
        return 0;
    }

    fn XXX<'r, 's>(_x: &mut CPU<'s>) -> u8 {
        return 0;
    }

    pub fn complete(&self) -> bool {
        return self.get_cycles() == 0;
    }
}
