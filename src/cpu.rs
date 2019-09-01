#![allow(non_snake_case)]
use crate::bus::Bus;

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
    pub bus: &'a Bus,
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
    pub clock_count: u32,
    lookup: [(&'static str, fn(&mut CPU) -> u8, fn(&mut CPU) -> u8, u8); 256],
}

impl<'a> CPU<'a> {
    pub fn new(b: &'a Bus) -> CPU<'a> {
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
            clock_count: 0,
            lookup: [
                ("BRK", CPU::BRK, CPU::IMM, 7),
                ("ORA", CPU::ORA, CPU::IZX, 6),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("ORA", CPU::ORA, CPU::ZP0, 3),
                ("ASL", CPU::ASL, CPU::ZP0, 5),
                ("SLO", CPU::SLO, CPU::ZP0, 5),
                ("PHP", CPU::PHP, CPU::IMP, 3),
                ("ORA", CPU::ORA, CPU::IMM, 2),
                ("ASL", CPU::ASL, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("NOP", CPU::NOP, CPU::ABS, 4),
                ("ORA", CPU::ORA, CPU::ABS, 4),
                ("ASL", CPU::ASL, CPU::ABS, 6),
                ("SLO", CPU::SLO, CPU::ABS, 6),
                ("BPL", CPU::BPL, CPU::REL, 2),
                ("ORA", CPU::ORA, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("ORA", CPU::ORA, CPU::ZPX, 4),
                ("ASL", CPU::ASL, CPU::ZPX, 6),
                ("SLO", CPU::SLO, CPU::ZPX, 6),
                ("CLC", CPU::CLC, CPU::IMP, 2),
                ("ORA", CPU::ORA, CPU::ABY, 4),
                ("???", CPU::NOP, CPU::IMP, 2),
                ("SLO", CPU::SLO, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("ORA", CPU::ORA, CPU::ABX, 4),
                ("ASL", CPU::ASL, CPU::ABX, 7),
                ("SLO", CPU::SLO, CPU::ABX, 7),
                ("JSR", CPU::JSR, CPU::ABS, 6),
                ("AND", CPU::AND, CPU::IZX, 6),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::IZX, 8),
                ("BIT", CPU::BIT, CPU::ZP0, 3),
                ("AND", CPU::AND, CPU::ZP0, 3),
                ("ROL", CPU::ROL, CPU::ZP0, 5),
                ("RLA", CPU::RLA, CPU::ZP0, 5),
                ("PLP", CPU::PLP, CPU::IMP, 4),
                ("AND", CPU::AND, CPU::IMM, 2),
                ("ROL", CPU::ROL, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("BIT", CPU::BIT, CPU::ABS, 4),
                ("AND", CPU::AND, CPU::ABS, 4),
                ("ROL", CPU::ROL, CPU::ABS, 6),
                ("RLA", CPU::RLA, CPU::ABS, 6),
                ("BMI", CPU::BMI, CPU::REL, 2),
                ("AND", CPU::AND, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("AND", CPU::AND, CPU::ZPX, 4),
                ("ROL", CPU::ROL, CPU::ZPX, 6),
                ("RLA", CPU::RLA, CPU::ZPX, 6),
                ("SEC", CPU::SEC, CPU::IMP, 2),
                ("AND", CPU::AND, CPU::ABY, 4),
                ("???", CPU::NOP, CPU::IMP, 2),
                ("RLA", CPU::RLA, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("AND", CPU::AND, CPU::ABX, 4),
                ("ROL", CPU::ROL, CPU::ABX, 7),
                ("RLA", CPU::RLA, CPU::ABX, 7),
                ("RTI", CPU::RTI, CPU::IMP, 6),
                ("EOR", CPU::EOR, CPU::IZX, 6),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("EOR", CPU::EOR, CPU::ZP0, 3),
                ("LSR", CPU::LSR, CPU::ZP0, 5),
                ("SRE", CPU::SRE, CPU::ZP0, 5),
                ("PHA", CPU::PHA, CPU::IMP, 3),
                ("EOR", CPU::EOR, CPU::IMM, 2),
                ("LSR", CPU::LSR, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("JMP", CPU::JMP, CPU::ABS, 3),
                ("EOR", CPU::EOR, CPU::ABS, 4),
                ("LSR", CPU::LSR, CPU::ABS, 6),
                ("SRE", CPU::SRE, CPU::ABS, 6),
                ("BVC", CPU::BVC, CPU::REL, 2),
                ("EOR", CPU::EOR, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("EOR", CPU::EOR, CPU::ZPX, 4),
                ("LSR", CPU::LSR, CPU::ZPX, 6),
                ("SRE", CPU::SRE, CPU::ZPX, 6),
                ("CLI", CPU::CLI, CPU::IMP, 2),
                ("EOR", CPU::EOR, CPU::ABY, 4),
                ("???", CPU::NOP, CPU::IMP, 2),
                ("SRE", CPU::SRE, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("EOR", CPU::EOR, CPU::ABX, 4),
                ("LSR", CPU::LSR, CPU::ABX, 7),
                ("SRE", CPU::SRE, CPU::ABX, 7),
                ("RTS", CPU::RTS, CPU::IMP, 6),
                ("ADC", CPU::ADC, CPU::IZX, 6),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("RRA", CPU::RRA, CPU::IZX, 8),
                ("NOP", CPU::NOP, CPU::ZP0, 3),
                ("ADC", CPU::ADC, CPU::ZP0, 3),
                ("ROR", CPU::ROR, CPU::ZP0, 5),
                ("RRA", CPU::RRA, CPU::ZP0, 5),
                ("PLA", CPU::PLA, CPU::IMP, 4),
                ("ADC", CPU::ADC, CPU::IMM, 2),
                ("ROR", CPU::ROR, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("JMP", CPU::JMP, CPU::IND, 5),
                ("ADC", CPU::ADC, CPU::ABS, 4),
                ("ROR", CPU::ROR, CPU::ABS, 6),
                ("RRA", CPU::RRA, CPU::ABS, 6),
                ("BVS", CPU::BVS, CPU::REL, 2),
                ("ADC", CPU::ADC, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("RRA", CPU::RRA, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("ADC", CPU::ADC, CPU::ZPX, 4),
                ("ROR", CPU::ROR, CPU::ZPX, 6),
                ("RRA", CPU::RRA, CPU::ZPX, 6),
                ("SEI", CPU::SEI, CPU::IMP, 2),
                ("ADC", CPU::ADC, CPU::ABY, 4),
                ("???", CPU::NOP, CPU::IMP, 2),
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
                ("???", CPU::XXX, CPU::IMP, 2),
                ("STY", CPU::STY, CPU::ABS, 4),
                ("STA", CPU::STA, CPU::ABS, 4),
                ("STX", CPU::STX, CPU::ABS, 4),
                ("SAX", CPU::SAX, CPU::ABS, 4),
                ("BCC", CPU::BCC, CPU::REL, 2),
                ("STA", CPU::STA, CPU::IZY, 6),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 6),
                ("STY", CPU::STY, CPU::ZPX, 4),
                ("STA", CPU::STA, CPU::ZPX, 4),
                ("STX", CPU::STX, CPU::ZPY, 4),
                ("SAX", CPU::SAX, CPU::ZPY, 4),
                ("TYA", CPU::TYA, CPU::IMP, 2),
                ("STA", CPU::STA, CPU::ABY, 5),
                ("TXS", CPU::TXS, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 5),
                ("???", CPU::NOP, CPU::IMP, 5),
                ("STA", CPU::STA, CPU::ABX, 5),
                ("???", CPU::XXX, CPU::IMP, 5),
                ("???", CPU::XXX, CPU::IMP, 5),
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
                ("???", CPU::XXX, CPU::IMP, 2),
                ("LDY", CPU::LDY, CPU::ABS, 4),
                ("LDA", CPU::LDA, CPU::ABS, 4),
                ("LDX", CPU::LDX, CPU::ABS, 4),
                ("LAX", CPU::LAX, CPU::ABS, 4),
                ("BCS", CPU::BCS, CPU::REL, 2),
                ("LDA", CPU::LDA, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("LAX", CPU::LAX, CPU::IZY, 5),
                ("LDY", CPU::LDY, CPU::ZPX, 4),
                ("LDA", CPU::LDA, CPU::ZPX, 4),
                ("LDX", CPU::LDX, CPU::ZPY, 4),
                ("LAX", CPU::LAX, CPU::ZPY, 4),
                ("CLV", CPU::CLV, CPU::IMP, 2),
                ("LDA", CPU::LDA, CPU::ABY, 4),
                ("TSX", CPU::TSX, CPU::IMP, 2),
                ("???", CPU::XXX, CPU::IMP, 4),
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
                ("???", CPU::XXX, CPU::IMP, 2),
                ("CPY", CPU::CPY, CPU::ABS, 4),
                ("CMP", CPU::CMP, CPU::ABS, 4),
                ("DEC", CPU::DEC, CPU::ABS, 6),
                ("DCP", CPU::DCP, CPU::ABS, 6),
                ("BNE", CPU::BNE, CPU::REL, 2),
                ("CMP", CPU::CMP, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
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
                ("ISC", CPU::ISC, CPU::IZX, 8),
                ("CPX", CPU::CPX, CPU::ZP0, 3),
                ("SBC", CPU::SBC, CPU::ZP0, 3),
                ("INC", CPU::INC, CPU::ZP0, 5),
                ("ISC", CPU::ISC, CPU::ZP0, 5),
                ("INX", CPU::INX, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::IMM, 2),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::IMM, 2),
                ("CPX", CPU::CPX, CPU::ABS, 4),
                ("SBC", CPU::SBC, CPU::ABS, 4),
                ("INC", CPU::INC, CPU::ABS, 6),
                ("ISC", CPU::ISC, CPU::ABS, 6),
                ("BEQ", CPU::BEQ, CPU::REL, 2),
                ("SBC", CPU::SBC, CPU::IZY, 5),
                ("???", CPU::XXX, CPU::IMP, 2),
                ("ISC", CPU::ISC, CPU::IZY, 8),
                ("NOP", CPU::NOP, CPU::ZPX, 4),
                ("SBC", CPU::SBC, CPU::ZPX, 4),
                ("INC", CPU::INC, CPU::ZPX, 6),
                ("ISC", CPU::ISC, CPU::ZPX, 6),
                ("SED", CPU::SED, CPU::IMP, 2),
                ("SBC", CPU::SBC, CPU::ABY, 4),
                ("NOP", CPU::NOP, CPU::IMP, 2),
                ("ISC", CPU::ISC, CPU::ABY, 7),
                ("NOP", CPU::NOP, CPU::ABX, 4),
                ("SBC", CPU::SBC, CPU::ABX, 4),
                ("INC", CPU::INC, CPU::ABX, 7),
                ("ISC", CPU::ISC, CPU::ABX, 7),
            ],
        }
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.read(addr)
    }

    fn write(&self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    pub fn reset(&mut self) {
        self.addr_abs = 0xFFFC;
        let lo: u16 = self.read(self.addr_abs + 0).into();
        let hi: u16 = self.read(self.addr_abs + 1).into();

        self.pc = (hi << 8) | lo;

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stkp = 0xFD;
        self.status = 0x00 | FLAGS::U as u8 | FLAGS::I as u8;

        self.addr_rel = 0x0000;
        self.addr_abs = 0x0000;
        self.fetched = 0x00;

        self.cycles = 8;
    }

    pub fn irq(&mut self) {
        if self.get_flag(FLAGS::I) == 0 {
            self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
            self.stkp -= 1;
            self.write(0x0100 + self.stkp as u16, (self.pc & 0x00FF) as u8);

            self.set_flag(FLAGS::B, false);
            self.set_flag(FLAGS::U, true);
            self.set_flag(FLAGS::I, true);
            self.write(0x100 + self.stkp as u16, self.status);
            self.stkp -= 1;

            self.addr_abs = 0xFFFE;
            let lo: u16 = self.read(self.addr_abs + 0) as u16;
            let hi: u16 = self.read(self.addr_abs + 1) as u16;
            self.pc = (hi << 8) | lo;

            self.cycles = 7;
        }
    }

    pub fn nmi(&mut self) {
        self.write(0x0100 + self.stkp as u16, ((self.pc >> 8) & 0x00FF) as u8);
        self.stkp -= 1;
        self.write(0x0100 + self.stkp as u16, (self.pc & 0x00FF) as u8);

        self.set_flag(FLAGS::B, false);
        self.set_flag(FLAGS::U, true);
        self.set_flag(FLAGS::I, true);
        self.write(0x100 + self.stkp as u16, self.status);
        self.stkp -= 1;

        self.addr_abs = 0xFFFA;
        let lo: u16 = self.read(self.addr_abs + 0) as u16;
        let hi: u16 = self.read(self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo;

        self.cycles = 8;
    }

    pub fn clock(&mut self) {
        self.cycles -= 1;
        if self.cycles == 0 {
            self.opcode = self.read(self.pc);
            println!(
                "{:04X} {:02X} {}   A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
                self.pc,
                self.opcode,
                self.lookup[self.opcode as usize].0,
                self.a,
                self.x,
                self.y,
                self.status,
                self.stkp
            );
            self.set_flag(FLAGS::U, true);
            self.pc += 1;
            self.cycles = self.lookup[self.opcode as usize].3;
            let additional_cycle1: u8 = self.lookup[self.opcode as usize].2(self);
            let additional_cycle2: u8 = self.lookup[self.opcode as usize].1(self);
            self.status &= 0xEF;
            self.cycles += additional_cycle1 & additional_cycle2;
            self.set_flag(FLAGS::U, true);
        }
        self.clock_count += 1;
    }

    fn get_flag(&self, f: FLAGS) -> u8 {
        if self.status & f as u8 > 0 {
            return 1;
        } else {
            return 0;
        }
    }

    fn set_flag(&mut self, f: FLAGS, v: bool) {
        if v {
            self.status |= f as u8;
        } else {
            self.status &= !(f as u8);
        }
    }

    fn IMP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetched = x.a;
        return 0;
    }

    fn IMM<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.addr_abs = x.pc;
        x.pc += 1;
        return 0;
    }

    fn ZP0<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.addr_abs = x.read(x.pc) as u16;
        x.pc += 1;
        x.addr_abs &= 0x00FF;
        return 0;
    }

    fn ZPX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.addr_abs = (x.read(x.pc).wrapping_add(x.x)) as u16;
        x.pc += 1;
        x.addr_abs &= 0x00FF;
        return 0;
    }

    fn ZPY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.addr_abs = (x.read(x.pc).wrapping_add(x.y)) as u16;
        x.pc += 1;
        return 0;
    }

    fn REL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.addr_rel = x.read(x.pc) as u16;
        x.pc += 1;
        if (x.addr_rel & 0x80) != 0 {
            x.addr_rel |= 0xFF00;
        }
        return 0;
    }

    fn ABS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.pc) as u16;
        x.pc = x.pc.wrapping_add(1);
        let hi = x.read(x.pc) as u16;
        x.pc = x.pc.wrapping_add(1);

        x.addr_abs = (hi << 8) | lo;

        return 0;
    }

    fn ABX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.pc) as u16;
        x.pc += 1;
        let hi = x.read(x.pc) as u16;
        x.pc += 1;

        x.addr_abs = (hi << 8) | lo;
        x.addr_abs = x.addr_abs.wrapping_add(x.x as u16);
        if (x.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn ABY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let lo = x.read(x.pc) as u16;
        x.pc += 1;
        let hi = x.read(x.pc) as u16;
        x.pc += 1;

        x.addr_abs = (hi << 8) | lo;
        x.addr_abs = x.addr_abs.wrapping_add(x.y as u16);

        if (x.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn IND<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let ptr_lo = x.read(x.pc) as u16;
        x.pc += 1;
        let ptr_hi = x.read(x.pc) as u16;
        x.pc += 1;

        let ptr = (ptr_hi << 8) | ptr_lo;

        if ptr_lo == 0x00FF {
            x.addr_abs = ((x.read(ptr & 0xFF00) as u16) << 8) | x.read(ptr + 0) as u16;
        } else {
            x.addr_abs = ((x.read(ptr + 1) as u16) << 8) | x.read(ptr + 0) as u16;
        }
        return 0;
    }

    fn IZX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let t = x.read(x.pc) as u16;
        x.pc += 1;
        let lo = x.read((t + x.x as u16) & 0x00FF) as u16;
        let hi = x.read((t + x.x as u16 + 1) & 0x00FF) as u16;

        x.addr_abs = (hi << 8) | lo;

        return 0;
    }

    fn IZY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let t = x.read(x.pc) as u16;
        x.pc += 1;

        let lo = x.read(t & 0x00FF) as u16;
        let hi = x.read((t.wrapping_add(1)) & 0x00FF) as u16;

        x.addr_abs = (hi << 8) | lo;
        x.addr_abs = x.addr_abs.wrapping_add(x.y as u16);

        if (x.addr_abs & 0xFF00) != (hi << 8) {
            return 1;
        } else {
            return 0;
        }
    }

    fn fetch(&mut self) {
        if self.lookup[self.opcode as usize].2 as usize != CPU::IMP as usize {
            self.fetched = self.read(self.addr_abs);
        }
        // return self.fetched;
    }

    fn ADC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();

        x.temp = (x.a as u16)
            .wrapping_add(x.fetched as u16)
            .wrapping_add(x.get_flag(FLAGS::C) as u16);
        x.set_flag(FLAGS::C, x.temp > 255);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0);
        x.set_flag(
            FLAGS::V,
            ((!(x.a as u16 ^ x.fetched as u16) & (x.a as u16 ^ x.temp as u16)) & 0x0080) != 0,
        );
        x.set_flag(FLAGS::N, (x.temp & 0x80) != 0);

        x.a = (x.temp & 0x00FF) as u8;
        return 1;
    }

    fn SBC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();

        let value = x.fetched as u16 ^ 0x00FF;

        x.temp = (x.a as u16)
            .wrapping_add(value)
            .wrapping_add(x.get_flag(FLAGS::C) as u16);
        x.set_flag(FLAGS::C, (x.temp & 0xFF00) != 0);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0);
        x.set_flag(
            FLAGS::V,
            ((x.temp ^ x.a as u16) & (x.temp ^ value) & 0x0080) != 0,
        );
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        x.a = (x.temp & 0x00FF) as u8;
        return 1;
    }

    fn AND<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.a &= x.fetched;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 1;
    }

    fn ASL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.fetched as u16) << 1;
        x.set_flag(FLAGS::C, (x.temp & 0xFF00) > 0);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.temp & 0x80) != 0);
        if x.lookup[x.opcode as usize].2 as usize == CPU::IMP as usize {
            x.a = (x.temp & 0x00FF) as u8;
        } else {
            x.write(x.addr_abs, (x.temp & 0x00FF) as u8)
        }
        return 0;
    }

    fn BCC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::C) == 0 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BCS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::C) == 1 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BEQ<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::Z) == 1 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BIT<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.a & x.fetched) as u16;
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.fetched & (1 << 7)) != 0);
        x.set_flag(FLAGS::V, (x.fetched & (1 << 6)) != 0);
        return 0;
    }

    fn BMI<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::N) == 1 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BNE<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::Z) == 0 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BPL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::N) == 0 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BRK<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.pc += 1;

        x.set_flag(FLAGS::I, true);
        x.write(
            (x.stkp as u16).wrapping_add(0x0100),
            ((x.pc >> 8) & 0x00ff) as u8,
        );
        x.stkp -= 1;
        x.write((x.stkp as u16).wrapping_add(0x0100), (x.pc & 0x00FF) as u8);
        x.stkp -= 1;

        x.set_flag(FLAGS::B, true);
        x.write((x.stkp as u16).wrapping_add(0x0100), x.status);
        x.stkp += 1;
        x.set_flag(FLAGS::B, false);

        x.pc = x.read(0xFFFE) as u16 | ((x.read(0xFFFF) as u16) << 8);
        return 0;
    }

    fn BVC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::V) == 0 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
        }
        return 0;
    }

    fn BVS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        if x.get_flag(FLAGS::V) == 1 {
            x.cycles += 1;
            x.addr_abs = x.pc.wrapping_add(x.addr_rel);

            if (x.addr_abs & 0xFF00) != (x.pc & 0xFF00) {
                x.cycles += 1;
            }

            x.pc = x.addr_abs;
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
        x.temp = (x.a as u16).wrapping_sub(x.fetched as u16);
        x.set_flag(FLAGS::C, x.a >= x.fetched);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        return 1;
    }

    fn CPX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.x as u16).wrapping_sub(x.fetched as u16);
        x.set_flag(FLAGS::C, x.x >= x.fetched);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        return 0;
    }

    fn CPY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.y as u16).wrapping_sub(x.fetched as u16);
        x.set_flag(FLAGS::C, x.y >= x.fetched);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        return 0;
    }

    fn DCP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::DEC(x);
        CPU::CMP(x);
        return 0;
    }

    fn DEC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.fetched as u16).wrapping_sub(1);
        x.write(x.addr_abs, (x.temp & 0x00FF) as u8);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        return 0;
    }

    fn DEX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.x = x.x.wrapping_sub(1);
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 0;
    }

    fn DEY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.y = x.y.wrapping_sub(1);
        x.set_flag(FLAGS::Z, x.y == 0x00);
        x.set_flag(FLAGS::N, (x.y & 0x80) != 0);
        return 0;
    }

    fn EOR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.a = x.a ^ x.fetched;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 1;
    }

    fn INC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.fetched.wrapping_add(1)) as u16;
        x.write(x.addr_abs, (x.temp & 0x00FF) as u8);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x00);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        return 0;
    }

    fn INX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.x = x.x.wrapping_add(1);
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 0;
    }

    fn INY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.y = x.y.wrapping_add(1);
        x.set_flag(FLAGS::Z, x.y == 0x00);
        x.set_flag(FLAGS::N, (x.y & 0x80) != 0);
        return 0;
    }

    fn ISC<'r, 's>(x: &mut CPU<'s>) -> u8 {
        CPU::INC(x);
        CPU::SBC(x);
        return 0;
    }

    fn JMP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.pc = x.addr_abs;
        return 0;
    }

    fn JSR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.pc -= 1;
        x.write(0x0100 + x.stkp as u16, ((x.pc >> 8) & 0x00FF) as u8);
        x.stkp = x.stkp.wrapping_sub(1);
        x.write(0x0100 + x.stkp as u16, (x.pc & 0x00FF) as u8);
        x.stkp = x.stkp.wrapping_sub(1);

        x.pc = x.addr_abs;
        return 0;
    }

    fn LAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.a = x.fetched;
        x.x = x.a;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 0;
    }

    fn LDA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.a = x.fetched;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 1;
    }

    fn LDX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.x = x.fetched;
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 1;
    }

    fn LDY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.y = x.fetched;
        x.set_flag(FLAGS::Z, x.y == 0x00);
        x.set_flag(FLAGS::N, (x.y & 0x80) != 0);
        return 1;
    }

    fn LSR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.set_flag(FLAGS::C, (x.fetched & 0x0001) != 0);
        x.temp = (x.fetched >> 1) as u16;
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        if x.lookup[x.opcode as usize].2 as usize == CPU::IMP as usize {
            x.a = (x.temp & 0x00FF) as u8;
        } else {
            x.write(x.addr_abs, (x.temp & 0x00FF) as u8);
        }
        return 0;
    }

    fn NOP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        let rc;
        match x.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => rc = 1,
            _ => rc = 0,
        }
        return rc;
    }

    fn ORA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.a = x.a | x.fetched;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 1;
    }

    fn PHA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write((x.stkp as u16).wrapping_add(0x0100), x.a);
        x.stkp = x.stkp.wrapping_sub(1);
        return 0;
    }

    fn PHP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(
            (x.stkp as u16).wrapping_add(0x0100),
            x.status | FLAGS::B as u8 | FLAGS::U as u8,
        );
        x.set_flag(FLAGS::B, false);
        x.set_flag(FLAGS::U, false);
        x.stkp = x.stkp.wrapping_sub(1);
        return 0;
    }

    fn PLA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.stkp = x.stkp.wrapping_add(1);
        x.a = x.read((x.stkp as u16).wrapping_add(0x0100));
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 0;
    }

    fn PLP<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.stkp += 1;
        x.status = x.read((x.stkp as u16).wrapping_add(0x0100));
        x.set_flag(FLAGS::U, true);
        return 0;
    }

    fn ROL<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = ((x.fetched as u16) << 1) | x.get_flag(FLAGS::C) as u16;
        x.set_flag(FLAGS::C, (x.temp & 0xFF00) != 0);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        if x.lookup[x.opcode as usize].2 as usize == CPU::IMP as usize {
            x.a = (x.temp & 0x00FF) as u8;
        } else {
            x.write(x.addr_abs, (x.temp & 0x00FF) as u8);
        }
        return 0;
    }

    fn ROR<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.fetch();
        x.temp = (x.get_flag(FLAGS::C) << 7) as u16 | (x.fetched >> 1) as u16;
        x.set_flag(FLAGS::C, (x.fetched & 0x01) != 0);
        x.set_flag(FLAGS::Z, (x.temp & 0x00FF) == 0x0000);
        x.set_flag(FLAGS::N, (x.temp & 0x0080) != 0);
        if x.lookup[x.opcode as usize].2 as usize == CPU::IMP as usize {
            x.a = (x.temp & 0x00FF) as u8;
        } else {
            x.write(x.addr_abs, (x.temp & 0x00FF) as u8);
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
        x.stkp = x.stkp.wrapping_add(1);
        x.status = x.read((x.stkp as u16).wrapping_add(0x0100));
        x.status &= !(FLAGS::B as u8);
        x.status &= !(FLAGS::U as u8);

        x.stkp = x.stkp.wrapping_add(1);
        x.pc = x.read((x.stkp as u16).wrapping_add(0x0100)) as u16;
        x.stkp = x.stkp.wrapping_add(1);
        x.pc |= (x.read((x.stkp as u16).wrapping_add(0x0100)) as u16) << 8;
        return 0;
    }

    fn RTS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.stkp = x.stkp.wrapping_add(1);
        x.pc = x.read((x.stkp as u16).wrapping_add(0x0100)) as u16;
        x.stkp = x.stkp.wrapping_add(1);
        x.pc |= (x.read((x.stkp as u16).wrapping_add(0x0100)) as u16) << 8;

        x.pc += 1;
        return 0;
    }

    fn SAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.addr_abs, x.a & x.x);
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
        x.write(x.addr_abs, x.a);
        return 0;
    }

    fn STX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.addr_abs, x.x);
        return 0;
    }

    fn STY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.write(x.addr_abs, x.y);
        return 0;
    }

    fn TAX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.x = x.a;
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 0;
    }

    fn TAY<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.y = x.a;
        x.set_flag(FLAGS::Z, x.y == 0x00);
        x.set_flag(FLAGS::N, (x.y & 0x80) != 0);
        return 0;
    }

    fn TSX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.x = x.stkp;
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 0;
    }

    fn TXA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.a = x.x;
        x.set_flag(FLAGS::Z, x.x == 0x00);
        x.set_flag(FLAGS::N, (x.x & 0x80) != 0);
        return 0;
    }

    fn TXS<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.stkp = x.x;
        return 0;
    }

    fn TYA<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.a = x.y;
        x.set_flag(FLAGS::Z, x.a == 0x00);
        x.set_flag(FLAGS::N, (x.a & 0x80) != 0);
        return 0;
    }

    fn XXX<'r, 's>(x: &mut CPU<'s>) -> u8 {
        x.cycles = 0;
        return 0;
    }

    pub fn complete(&self) -> bool {
        return self.cycles == 0;
    }
}
