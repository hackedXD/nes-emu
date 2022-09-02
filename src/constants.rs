use lazy_static::lazy_static;
use std::collections::HashMap;
use crate::cpu;

#[derive(Clone)]
pub struct Status {
    pub carry: bool,
    pub zero: bool,
    pub interrupt: bool,
    pub decimal: bool,
    pub break_command: bool,
    pub unused: bool,
    pub overflow: bool,
    pub negative: bool,
}

impl Status {
    pub fn new() -> Status {
        Status {
            carry: false,
            zero: false,
            interrupt: false,
            decimal: false,
            break_command: false,
            unused: false,
            overflow: false,
            negative: false,
        }
    }

    pub fn to_byte(&self) -> u8 {
        let mut result = 0x00;
        result |= (self.carry as u8) << 0;
        result |= (self.zero as u8) << 1;
        result |= (self.interrupt as u8) << 2;
        result |= (self.decimal as u8) << 3;
        result |= (self.break_command as u8) << 4;
        result |= (self.unused as u8) << 5;
        result |= (self.overflow as u8) << 6;
        result |= (self.negative as u8) << 7;
        return result;
    }

    pub fn from_byte(input: u8) -> Status {
        let mut result = Status::new();
        result.carry = (input & 0b0000_0001) != 0;
        result.zero = (input & 0b0000_0010) != 0;
        result.interrupt = (input & 0b0000_0100) != 0;
        result.decimal = (input & 0b0000_1000) != 0;
        result.break_command = (input & 0b0001_0000) != 0;
        result.unused = (input & 0b0010_0000) != 0;
        result.overflow = (input & 0b0100_0000) != 0;
        result.negative = (input & 0b1000_0000) != 0;
        return result;
    }
}


type OpcodeOperation = fn(&mut cpu::CPU, AddressingMode);
pub struct OpCode {
    pub name: String,
    pub opcode: u8,
    pub addressing_mode: AddressingMode,
    pub bytes: u8,
    pub cycles: u8,
    pub operation: OpcodeOperation,
}

impl OpCode {
    pub fn new(name: &str, opcode: u8, addressing_mode: AddressingMode, bytes: u8, cycles: u8, operation: OpcodeOperation) -> OpCode {
        OpCode {
            name: name.to_string(),
            opcode: opcode,
            addressing_mode: addressing_mode,
            bytes: bytes,
            cycles: cycles,
            operation: operation,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
}

lazy_static! {
    pub static ref OPCODES: HashMap<u8, OpCode> = {
        let opcode_array = [
            OpCode::new("ADC", 0x69, AddressingMode::Immediate, 2, 2, cpu::CPU::ADC),
            OpCode::new("ADC", 0x65, AddressingMode::ZeroPage, 2, 3, cpu::CPU::ADC),
            OpCode::new("ADC", 0x75, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::ADC),
            OpCode::new("ADC", 0x6D, AddressingMode::Absolute, 3, 4, cpu::CPU::ADC),
            OpCode::new("ADC", 0x7D, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::ADC),
            OpCode::new("ADC", 0x79, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::ADC),
            OpCode::new("ADC", 0x61, AddressingMode::IndirectX, 2, 6, cpu::CPU::ADC),
            OpCode::new("ADC", 0x71, AddressingMode::IndirectY, 2, 5, cpu::CPU::ADC),
        
            OpCode::new("AND", 0x29, AddressingMode::Immediate, 2,  2, cpu::CPU::AND),
            OpCode::new("AND", 0x25, AddressingMode::ZeroPage, 2, 3, cpu::CPU::AND),
            OpCode::new("AND", 0x35, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::AND),
            OpCode::new("AND", 0x2D, AddressingMode::Absolute, 3, 4, cpu::CPU::AND),
            OpCode::new("AND", 0x3D, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::AND),
            OpCode::new("AND", 0x39, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::AND),
            OpCode::new("AND", 0x21, AddressingMode::IndirectX, 2, 6, cpu::CPU::AND),
            OpCode::new("AND", 0x31, AddressingMode::IndirectY, 2, 5, cpu::CPU::AND),
            
            OpCode::new("ASL", 0x0A, AddressingMode::Accumulator, 1, 2, cpu::CPU::ASL),
            OpCode::new("ASL", 0x06, AddressingMode::ZeroPage, 2, 5, cpu::CPU::ASL),
            OpCode::new("ASL", 0x16, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::ASL),
            OpCode::new("ASL", 0x0E, AddressingMode::Absolute, 3, 6, cpu::CPU::ASL),
            OpCode::new("ASL", 0x1E, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::ASL),
        
            OpCode::new("BCC", 0x90, AddressingMode::Relative, 2, 2, cpu::CPU::BCC),
        
            OpCode::new("BCS", 0xB0, AddressingMode::Relative, 2, 2, cpu::CPU::BCS),
        
            OpCode::new("BEQ", 0xF0, AddressingMode::Relative, 2, 2, cpu::CPU::BEQ),
        
            OpCode::new("BIT", 0x24, AddressingMode::ZeroPage, 2, 3, cpu::CPU::BIT),
            OpCode::new("BIT", 0x2C, AddressingMode::Absolute, 3, 4, cpu::CPU::BIT),
        
            OpCode::new("BMI", 0x30, AddressingMode::Relative, 2, 2, cpu::CPU::BMI),
        
            OpCode::new("BNE", 0xD0, AddressingMode::Relative, 2, 2, cpu::CPU::BNE),
        
            OpCode::new("BPL", 0x10, AddressingMode::Relative, 2, 2, cpu::CPU::BPL),
        
            OpCode::new("BRK", 0x00, AddressingMode::Implicit, 1, 7, cpu::CPU::BRK),
        
            OpCode::new("BVC", 0x50, AddressingMode::Relative, 2, 2, cpu::CPU::BVC),
        
            OpCode::new("BVS", 0x70, AddressingMode::Relative, 2, 2, cpu::CPU::BVS),
        
            OpCode::new("CLC", 0x18, AddressingMode::Implicit, 1, 2, cpu::CPU::CLC),
        
            OpCode::new("CLD", 0xD8, AddressingMode::Implicit, 1, 2, cpu::CPU::CLD),
        
            OpCode::new("CLI", 0x58, AddressingMode::Implicit, 1, 2, cpu::CPU::CLI),
        
            OpCode::new("CLV", 0xB8, AddressingMode::Implicit, 1, 2, cpu::CPU::CLV),
        
            OpCode::new("CMP", 0xC9, AddressingMode::Immediate, 2, 2, cpu::CPU::CMP),
            OpCode::new("CMP", 0xC5, AddressingMode::ZeroPage, 2, 3, cpu::CPU::CMP),
            OpCode::new("CMP", 0xD5, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::CMP),
            OpCode::new("CMP", 0xCD, AddressingMode::Absolute, 3, 4, cpu::CPU::CMP),
            OpCode::new("CMP", 0xDD, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::CMP),
            OpCode::new("CMP", 0xD9, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::CMP),
            OpCode::new("CMP", 0xC1, AddressingMode::IndirectX, 2, 6, cpu::CPU::CMP),
            OpCode::new("CMP", 0xD1, AddressingMode::IndirectY, 2, 5, cpu::CPU::CMP),
        
            OpCode::new("CPX", 0xE0, AddressingMode::Immediate, 2,  2, cpu::CPU::CPX),
            OpCode::new("CPX", 0xE4, AddressingMode::ZeroPage, 2, 3, cpu::CPU::CPX),
            OpCode::new("CPX", 0xEC, AddressingMode::Absolute, 3, 4, cpu::CPU::CPX),
        
            OpCode::new("CPY", 0xC0, AddressingMode::Immediate, 2, 2, cpu::CPU::CPY),
            OpCode::new("CPY", 0xC4, AddressingMode::ZeroPage, 2, 3, cpu::CPU::CPY),
            OpCode::new("CPY", 0xCC, AddressingMode::Absolute, 3,  4, cpu::CPU::CPY),
        
            OpCode::new("DEC", 0xC6, AddressingMode::ZeroPage, 2, 5, cpu::CPU::DEC),
            OpCode::new("DEC", 0xD6, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::DEC),
            OpCode::new("DEC", 0xCE, AddressingMode::Absolute, 3, 6, cpu::CPU::DEC),
            OpCode::new("DEC", 0xDE, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::DEC),
        
            OpCode::new("DEX", 0xCA, AddressingMode::Implicit, 1, 2, cpu::CPU::DEX),
            
            OpCode::new("DEY", 0x88, AddressingMode::Implicit, 1,  2, cpu::CPU::DEY),
        
            OpCode::new("EOR", 0x49, AddressingMode::Immediate, 2, 2, cpu::CPU::EOR),
            OpCode::new("EOR", 0x45, AddressingMode::ZeroPage, 2, 3, cpu::CPU::EOR),
            OpCode::new("EOR", 0x55, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::EOR),
            OpCode::new("EOR", 0x4D, AddressingMode::Absolute, 3, 4, cpu::CPU::EOR),
            OpCode::new("EOR", 0x5D, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::EOR),
            OpCode::new("EOR", 0x59, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::EOR),
            OpCode::new("EOR", 0x41, AddressingMode::IndirectX, 2, 6, cpu::CPU::EOR),
            OpCode::new("EOR", 0x51, AddressingMode::IndirectY, 2, 5, cpu::CPU::EOR),
        
            OpCode::new("INC", 0xE6, AddressingMode::ZeroPage, 2, 5, cpu::CPU::INC),
            OpCode::new("INC", 0xF6, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::INC),
            OpCode::new("INC", 0xEE, AddressingMode::Absolute, 3, 6, cpu::CPU::INC),
            OpCode::new("INC", 0xFE, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::INC),
        
            OpCode::new("INX", 0xE8, AddressingMode::Implicit, 1, 2, cpu::CPU::INX),
        
            OpCode::new("INY", 0xC8, AddressingMode::Implicit, 1, 2, cpu::CPU::INY),
        
            OpCode::new("JMP", 0x4C, AddressingMode::Absolute, 3, 3, cpu::CPU::JMP),
            OpCode::new("JMP", 0x6C, AddressingMode::Indirect, 3, 5, cpu::CPU::JMP),
        
            OpCode::new("JSR", 0x20, AddressingMode::Absolute, 3, 6, cpu::CPU::JSR),
        
            OpCode::new("LDA", 0xA9, AddressingMode::Immediate, 2, 2, cpu::CPU::LDA),
            OpCode::new("LDA", 0xA5, AddressingMode::ZeroPage, 2, 3, cpu::CPU::LDA),
            OpCode::new("LDA", 0xB5, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::LDA),
            OpCode::new("LDA", 0xAD, AddressingMode::Absolute, 3, 4, cpu::CPU::LDA),
            OpCode::new("LDA", 0xBD, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::LDA),
            OpCode::new("LDA", 0xB9, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::LDA),
            OpCode::new("LDA", 0xA1, AddressingMode::IndirectX, 2, 6, cpu::CPU::LDA),
            OpCode::new("LDA", 0xB1, AddressingMode::IndirectY, 2, 5, cpu::CPU::LDA),
        
            OpCode::new("LDX", 0xA2, AddressingMode::Immediate, 2, 2, cpu::CPU::LDX),
            OpCode::new("LDX", 0xA6, AddressingMode::ZeroPage, 2, 3, cpu::CPU::LDX),
            OpCode::new("LDX", 0xB6, AddressingMode::ZeroPageY, 2, 4, cpu::CPU::LDX),
            OpCode::new("LDX", 0xAE, AddressingMode::Absolute, 3, 4, cpu::CPU::LDX),
            OpCode::new("LDX", 0xBE, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::LDX),
        
            OpCode::new("LDY", 0xA0, AddressingMode::Immediate, 2, 2, cpu::CPU::LDY),
            OpCode::new("LDY", 0xA4, AddressingMode::ZeroPage, 2, 3, cpu::CPU::LDY),
            OpCode::new("LDY", 0xB4, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::LDY),
            OpCode::new("LDY", 0xAC, AddressingMode::Absolute, 3, 4, cpu::CPU::LDY),
            OpCode::new("LDY", 0xBC, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::LDY),
        
            OpCode::new("LSR", 0x4A, AddressingMode::Accumulator, 1, 2, cpu::CPU::LSR),
            OpCode::new("LSR", 0x46, AddressingMode::ZeroPage, 2, 5, cpu::CPU::LSR),
            OpCode::new("LSR", 0x56, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::LSR),
            OpCode::new("LSR", 0x4E, AddressingMode::Absolute, 3, 6, cpu::CPU::LSR),
            OpCode::new("LSR", 0x5E, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::LSR),
        
            OpCode::new("NOP", 0xEA, AddressingMode::Implicit, 1, 2, cpu::CPU::NOP),
        
            OpCode::new("ORA", 0x09, AddressingMode::Immediate, 2, 2, cpu::CPU::ORA),
            OpCode::new("ORA", 0x05, AddressingMode::ZeroPage, 2, 3, cpu::CPU::ORA),
            OpCode::new("ORA", 0x15, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::ORA),
            OpCode::new("ORA", 0x0D, AddressingMode::Absolute, 3, 4, cpu::CPU::ORA),
            OpCode::new("ORA", 0x1D, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::ORA),
            OpCode::new("ORA", 0x19, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::ORA),
            OpCode::new("ORA", 0x01, AddressingMode::IndirectX, 2, 6, cpu::CPU::ORA),
            OpCode::new("ORA", 0x11, AddressingMode::IndirectY, 2, 5, cpu::CPU::ORA),
        
            OpCode::new("PHA", 0x48, AddressingMode::Implicit, 1, 3, cpu::CPU::PHA),
        
            OpCode::new("PHP", 0x08, AddressingMode::Implicit, 1, 3, cpu::CPU::PHP),
        
            OpCode::new("PLA", 0x68, AddressingMode::Implicit, 1, 4, cpu::CPU::PLA),
        
            OpCode::new("PLP", 0x28, AddressingMode::Implicit, 1, 4, cpu::CPU::PLP),
        
            OpCode::new("ROL", 0x2A, AddressingMode::Accumulator, 1, 2, cpu::CPU::ROL),
            OpCode::new("ROL", 0x26, AddressingMode::ZeroPage, 2, 5, cpu::CPU::ROL),
            OpCode::new("ROL", 0x36, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::ROL),
            OpCode::new("ROL", 0x2E, AddressingMode::Absolute, 3, 6, cpu::CPU::ROL),
            OpCode::new("ROL", 0x3E, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::ROL),
        
            OpCode::new("ROR", 0x6A, AddressingMode::Accumulator, 1, 2, cpu::CPU::ROR),
            OpCode::new("ROR", 0x66, AddressingMode::ZeroPage, 2, 5, cpu::CPU::ROR),
            OpCode::new("ROR", 0x76, AddressingMode::ZeroPageX, 2, 6, cpu::CPU::ROR),
            OpCode::new("ROR", 0x6E, AddressingMode::Absolute, 3, 6, cpu::CPU::ROR),
            OpCode::new("ROR", 0x7E, AddressingMode::AbsoluteX, 3, 7, cpu::CPU::ROR),
        
            OpCode::new("RTI", 0x40, AddressingMode::Implicit, 1, 6, cpu::CPU::RTI),
        
            OpCode::new("RTS", 0x60, AddressingMode::Implicit, 1, 6, cpu::CPU::RTS),
        
            OpCode::new("SBC", 0xE9, AddressingMode::Immediate, 2, 2, cpu::CPU::SBC),
            OpCode::new("SBC", 0xE5, AddressingMode::ZeroPage, 2, 3, cpu::CPU::SBC),
            OpCode::new("SBC", 0xF5, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::SBC),
            OpCode::new("SBC", 0xED, AddressingMode::Absolute, 3, 4, cpu::CPU::SBC),
            OpCode::new("SBC", 0xFD, AddressingMode::AbsoluteX, 3, 4, cpu::CPU::SBC),
            OpCode::new("SBC", 0xF9, AddressingMode::AbsoluteY, 3, 4, cpu::CPU::SBC),
            OpCode::new("SBC", 0xE1, AddressingMode::IndirectX, 2, 6, cpu::CPU::SBC),
            OpCode::new("SBC", 0xF1, AddressingMode::IndirectY, 2, 5, cpu::CPU::SBC),
        
            OpCode::new("SEC", 0x38, AddressingMode::Implicit, 1, 2, cpu::CPU::SEC),
        
            OpCode::new("SED", 0xF8, AddressingMode::Implicit, 1, 2, cpu::CPU::SED),
        
            OpCode::new("SEI", 0x78, AddressingMode::Implicit, 1, 2, cpu::CPU::SEI),
        
            OpCode::new("STA", 0x85, AddressingMode::ZeroPage, 2, 3, cpu::CPU::STA),
            OpCode::new("STA", 0x95, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::STA),
            OpCode::new("STA", 0x8D, AddressingMode::Absolute, 3, 4, cpu::CPU::STA),
            OpCode::new("STA", 0x9D, AddressingMode::AbsoluteX, 3, 5, cpu::CPU::STA),
            OpCode::new("STA", 0x99, AddressingMode::AbsoluteY, 3, 5, cpu::CPU::STA),
            OpCode::new("STA", 0x81, AddressingMode::IndirectX, 2, 6, cpu::CPU::STA),
            OpCode::new("STA", 0x91, AddressingMode::IndirectY, 2, 6, cpu::CPU::STA),
        
            OpCode::new("STX", 0x86, AddressingMode::ZeroPage, 2, 3, cpu::CPU::STX),
            OpCode::new("STX", 0x96, AddressingMode::ZeroPageY, 2, 4, cpu::CPU::STX),
            OpCode::new("STX", 0x8E, AddressingMode::Absolute, 3, 4, cpu::CPU::STX),
        
            OpCode::new("STY", 0x84, AddressingMode::ZeroPage, 2, 3, cpu::CPU::STY),
            OpCode::new("STY", 0x94, AddressingMode::ZeroPageX, 2, 4, cpu::CPU::STY),
            OpCode::new("STY", 0x8C, AddressingMode::Absolute, 3, 4, cpu::CPU::STY),
        
            OpCode::new("TAX", 0xAA, AddressingMode::Implicit, 1, 2, cpu::CPU::TAX),
        
            OpCode::new("TAY", 0xA8, AddressingMode::Implicit, 1, 2, cpu::CPU::TAY),
        
            OpCode::new("TSX", 0xBA, AddressingMode::Implicit, 1, 2, cpu::CPU::TSX),
        
            OpCode::new("TXA", 0x8A, AddressingMode::Implicit, 1, 2, cpu::CPU::TXA),
        
            OpCode::new("TXS", 0x9A, AddressingMode::Implicit, 1,  2, cpu::CPU::TXS),
        
            OpCode::new("TYA", 0x98, AddressingMode::Implicit, 1,  2, cpu::CPU::TYA),
        
        ];
        
        let mut opcode_map: HashMap::<u8, OpCode> = HashMap::new();

        for opc in opcode_array {
            opcode_map.insert(opc.opcode, opc);
        }
        
        opcode_map
    };
}