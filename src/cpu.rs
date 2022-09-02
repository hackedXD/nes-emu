use crate::bus::Bus;
use crate::constants::{
    AddressingMode,
    Status,
    OPCODES,
    OpCode
};


pub struct CPU {
    pub bus: Bus,
    pub status: Status,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub stack_pointer: u8,
    pub program_counter: u16,
    pub complete: bool,

    pub cycles: u64,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        CPU {
            bus: bus,
            status: Status::from_byte(0b100100),
            a: 0x00,
            x: 0x00,
            y: 0x00,
            stack_pointer: 0xFD,
            program_counter: 0x0000,
            cycles: 0,
            complete: false,
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        return self.bus.read(addr, false);
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data);
    }

    pub fn print_instruction(&mut self, opcode: &OpCode) {
        print!("${:04X}\t", self.program_counter);

        for i in 0..opcode.bytes {
            print!("{:02X} ", self.read(self.program_counter + i as u16));
        }

        print!("{}\t", "   ".repeat(3 - opcode.bytes as usize));

        print!("{} ", opcode.name);

        match opcode.addressing_mode {
            AddressingMode::Immediate => {
                print!("#${:02X}", self.read(self.program_counter + 1));
            },
            AddressingMode::ZeroPage => {
                print!("${:02X}", self.read(self.program_counter + 1));
            },
            AddressingMode::ZeroPageX => {
                print!("${:02X},X", self.read(self.program_counter + 1));
            },
            AddressingMode::ZeroPageY => {
                print!("${:02X},Y", self.read(self.program_counter + 1));
            },
            AddressingMode::Absolute => {
                print!("${:02X}{:02X}", self.read(self.program_counter + 2), self.read(self.program_counter + 1));
            }
            AddressingMode::AbsoluteX => {
                print!("${:02X}{:02X},X", self.read(self.program_counter + 2), self.read(self.program_counter + 1));
            }
            AddressingMode::AbsoluteY => {
                print!("${:02X}{:02X},Y", self.read(self.program_counter + 2), self.read(self.program_counter + 1));
            }
            AddressingMode::Indirect => {
                print!("(${:02X}{:02X})", self.read(self.program_counter + 2), self.read(self.program_counter + 1));
            }
            AddressingMode::IndirectX => {
                print!("(${:02X},X)", self.read(self.program_counter + 1));
            }
            AddressingMode::IndirectY => {
                print!("(${:02X}),Y", self.read(self.program_counter + 1));
            }
            AddressingMode::Relative => {
                print!("*{:+}", self.read(self.program_counter + 1) as i8);
            }
            AddressingMode::Accumulator => {
                print!("A");
            }
            AddressingMode::Implicit => {
                print!("");
            }
        }
        
    }

    pub fn clock(&mut self) {
        if self.cycles == 0 {
            let opcode = self.read(self.program_counter);
            match OPCODES.get(&opcode) {
                Some(op) => {
                    // self.print_instruction(&op);
                    self.program_counter += 1;
                    self.cycles = op.cycles as u64;
                    let pg_state = self.program_counter;

                    let operation = op.operation;
                    operation(self, op.addressing_mode);

                    if self.program_counter == pg_state {
                        self.program_counter += (op.bytes as u16) - 1;
                    }

                    // println!("");
                },
                None => {
                    println!("FAILED AT OP: {:02X} AND PC: {:04X}", opcode, self.program_counter);
                    panic!("Invalid opcode: 0x{:X}", opcode);
                }
            }
        }

        self.cycles -= 1;
    }

    pub fn load(&mut self, program: &Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.write(0x0600 + i, program[i as usize]);
        }
        self.write(0xFFFC, 0x00);
        self.write(0xFFFD, 0x06);
    }

    pub fn reset(&mut self) {
        let low = self.read(0xFFFC);
        let high = self.read(0xFFFD);

        self.program_counter = self.hilo_to_u16(high, low);

        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stack_pointer = 0xFD;

        self.cycles = 8;
    }

    pub fn nmi(&mut self) {
        self.stack_push((self.program_counter >> 8) as u8);
        self.stack_push(self.program_counter as u8);

        self.status.break_command = false;
        self.status.unused = true;
        self.status.interrupt = true;

        self.stack_push(self.status.to_byte());

        let low = self.read(0xFFFA);
        let high = self.read(0xFFFB);
        self.program_counter = self.hilo_to_u16(high, low);

        self.cycles = 8;
    }

    pub fn irq(&mut self) {
        if !self.status.interrupt {
            self.stack_push((self.program_counter >> 8) as u8);
            self.stack_push(self.program_counter as u8);

            self.status.break_command = false;
            self.status.unused = true;
            self.status.interrupt = true;

            self.stack_push(self.status.to_byte());

            let low = self.read(0xFFFE);
            let high = self.read(0xFFFF);
            self.program_counter = self.hilo_to_u16(high, low);

            self.cycles = 7;
        }
    }

    // ADDRESSING MODE
    fn get_address(&mut self, addressing_mode: AddressingMode) -> (u16, bool) {
        match addressing_mode {
            AddressingMode::Implicit => (0, false),
            AddressingMode::Accumulator => (0, false),
            AddressingMode::Immediate => (self.program_counter, false),
            AddressingMode::ZeroPage => (self.read(self.program_counter) as u16, false),
            AddressingMode::ZeroPageX => (self.read(self.program_counter)
                                            .wrapping_add(self.x) as u16, false),
            AddressingMode::ZeroPageY => (self.read(self.program_counter)
                                            .wrapping_add(self.y) as u16, false),
            AddressingMode::Relative => (0, false),
            AddressingMode::Absolute => {
                let low = self.read(self.program_counter);
                let high = self.read(self.program_counter + 1);

                (self.hilo_to_u16(high, low), false)
            },
            AddressingMode::AbsoluteX => {
                let low = self.read(self.program_counter);
                let high = self.read(self.program_counter + 1);

                let address = self.hilo_to_u16(high, low).wrapping_add(self.x as u16);

                (address, self.page_cross(address, (high as u16) << 8))
            },
            AddressingMode::AbsoluteY => {
                let low = self.read(self.program_counter);
                let high = self.read(self.program_counter + 1);

                let address = self.hilo_to_u16(high, low).wrapping_add(self.y as u16);

                (address, self.page_cross(address, (high as u16) << 8))
            },
            AddressingMode::Indirect => {
                let pointer_low = self.read(self.program_counter);
                let pointer_high = self.read(self.program_counter + 1);

                let pointer = self.hilo_to_u16(pointer_high, pointer_low);

                let low = self.read(pointer);
                let high = self.read(pointer + 1);

                (self.hilo_to_u16(high, low), false)
            },
            AddressingMode::IndirectX => {
                let pointer = self.read(self.program_counter)
                    .wrapping_add(self.x);

                let low = self.read(pointer as u16);
                let high = self.read((pointer + 1) as u16);

                (self.hilo_to_u16(high, low), false)
            },
            AddressingMode::IndirectY => {
                let pointer = self.read(self.program_counter) as u16;

                let low = self.read(pointer);
                let high = self.read(pointer + 1);

                let address = self.hilo_to_u16(high, low).wrapping_add(self.y as u16);

                (address, self.page_cross(address, (high as u16) << 8))
            },
        }
    }

    fn get_data(&mut self, addressing_mode: AddressingMode) -> (u8, bool) {
        if !((addressing_mode == AddressingMode::Implicit) || (addressing_mode == AddressingMode::Accumulator)) {
            let (address, page_boundary_cross) = self.get_address(addressing_mode);

            return (self.read(address), page_boundary_cross);
        } else {
            return (self.a, false);
        }
    }

    // UTILS
    fn update_zero_and_negative_flags(&mut self, register: u8) {
        self.status.zero = register == 0;
        self.status.negative = register >> 7 != 0;
    }

    fn cycle_if(&mut self, addressing_mode: AddressingMode, needed_addressing_mode: AddressingMode, condition: bool) {
        if needed_addressing_mode == addressing_mode && condition {
            self.cycles += 1;
        }
    }

    fn indexed_cycles(&mut self, addressing_mode: AddressingMode, page_boundary_crossed: bool) {
        self.cycle_if(addressing_mode, AddressingMode::AbsoluteX, page_boundary_crossed);
        self.cycle_if(addressing_mode, AddressingMode::AbsoluteY, page_boundary_crossed);
        self.cycle_if(addressing_mode, AddressingMode::IndirectY, page_boundary_crossed);
    }

    fn page_cross(&self, input1: u16, input2: u16) -> bool {
        (input1 & 0xFF00) != (input2 & 0xFF00)
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        // print!("READING: 0x{:02X} FROM SP: 0x{:02X} ", self.read(0x0100 + (self.stack_pointer as u16)), 0x0100 + (self.stack_pointer as u16));
        self.read(0x0100 + (self.stack_pointer as u16))
    }

    fn stack_push(&mut self, data: u8) {
        self.write(0x0100 + (self.stack_pointer as u16), data);
        // print!(" WRITING: 0x{:02X} TO SP: 0x{:02X} ", data, 0x0100 + (self.stack_pointer as u16));
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn disassemble(program: &Vec<u8>) {
        println!("Address\t\tHexdump\t\tDissassembly");
        println!("-------------------------------");
        // let mut pc: u16 = 0x0600;
        let mut i: u16 = 0;
        
        loop {
            print!("${:04X}\t\t", 0x0600 + i);
            
            let opc = &OPCODES[&program[i as usize]];

            for hex in 0..opc.bytes {
                print!("{:02X} ", program[(i as usize) + (hex as usize)]);
            }

            print!("{}", "    ".repeat(3 - opc.bytes as usize));

            print!("\t{} ", opc.name);

            match opc.addressing_mode {
                AddressingMode::Immediate => {
                    print!("#${:02X}", program[(i as usize) + 1]);
                },
                AddressingMode::ZeroPage => {
                    print!("${:02X}", program[(i as usize) + 1]);
                },
                AddressingMode::ZeroPageX => {
                    print!("${:02X},X", program[(i as usize) + 1]);
                },
                AddressingMode::ZeroPageY => {
                    print!("${:02X},Y", program[(i as usize) + 1]);
                },
                AddressingMode::Absolute => {
                    print!("${:02X}{:02X}", program[(i as usize) + 2], program[(i as usize) + 1]);
                }
                AddressingMode::AbsoluteX => {
                    print!("${:02X}{:02X},X", program[(i as usize) + 2], program[(i as usize) + 1]);
                }
                AddressingMode::AbsoluteY => {
                    print!("${:02X}{:02X},Y", program[(i as usize) + 2], program[(i as usize) + 1]);
                }
                AddressingMode::Indirect => {
                    print!("(${:02X}{:02X})", program[(i as usize) + 2], program[(i as usize) + 1]);
                }
                AddressingMode::IndirectX => {
                    print!("(${:02X},X)", program[(i as usize) + 1]);
                }
                AddressingMode::IndirectY => {
                    print!("(${:02X}),Y", program[(i as usize) + 1]);
                }
                AddressingMode::Relative => {
                    print!("*{:+}", program[(i as usize) + 1] as i8);
                }
                AddressingMode::Accumulator => {
                    print!("A");
                }
                AddressingMode::Implicit => {
                    print!("");
                }
            }

            i += opc.bytes as u16;
            println!("");

            if i as usize == program.len() {
                break;
            }
        }
    }

    fn hilo_to_u16(&self, high: u8, low: u8) -> u16 {
        (high as u16) << 8 | low as u16
    }

    // COMMONS
    fn branch(&mut self, _addressing_mode: AddressingMode, condition: bool) {
        if condition {
            let jump = self.read(self.program_counter) as i8;
            
            self.cycles += 1;

            let old_pc = self.program_counter;
            self.program_counter = self.program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.cycles += (self.page_cross(old_pc, self.program_counter) as u64) * 2;
        }
    }

    // ----------
    // OPERATIONS
    // ----------


    // MEMORY OPERATIONS
    #[allow(non_snake_case)]
    pub fn LDA(&mut self, addressing_mode: AddressingMode) { // Load Accumulator
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.a = value;
        
        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.update_zero_and_negative_flags(self.a);
    }

    #[allow(non_snake_case)]
    pub fn LDX(&mut self, addressing_mode: AddressingMode) { // Load X Register
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.x = value;

        self.cycle_if(addressing_mode, AddressingMode::AbsoluteY, page_boundary_cross);
        
        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn LDY(&mut self, addressing_mode: AddressingMode) { // Load Y Register
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.y = value;

        self.cycle_if(addressing_mode, AddressingMode::AbsoluteX, page_boundary_cross);
    
        self.update_zero_and_negative_flags(self.y);
    }

    #[allow(non_snake_case)]
    pub fn STA(&mut self, addressing_mode: AddressingMode) { // Store Accumulator
        let value = self.get_address(addressing_mode).0;

        self.write(value, self.a);
    }

    #[allow(non_snake_case)]
    pub fn STX(&mut self, addressing_mode: AddressingMode) { // Store X Register
        let value = self.get_address(addressing_mode).0;

        self.write(value, self.x);
    }

    #[allow(non_snake_case)]
    pub fn STY(&mut self, addressing_mode: AddressingMode) { // Store Y Register
        let value = self.get_address(addressing_mode).0;

        self.write(value, self.y);
    }


    // REGISTER TRANSFER OPERATIONS
    #[allow(non_snake_case)]
    pub fn TAX(&mut self, _addressing_mode: AddressingMode) { // Transfer Accumulator to X
        self.x = self.a;

        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn TAY(&mut self, _addressing_mode: AddressingMode) { // Transfer Accumulator to Y
        self.y = self.a;

        self.update_zero_and_negative_flags(self.y);
    }

    #[allow(non_snake_case)]
    pub fn TSX(&mut self, _addressing_mode: AddressingMode) { // Transfer Stack Pointer to X
        self.x = self.stack_pointer;
        
        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn TXA(&mut self, _addressing_mode: AddressingMode) { // Transfer X to Accumulator
        self.a = self.x;

        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn TXS(&mut self, _addressing_mode: AddressingMode) { // Transfer X to Stack Pointer
        self.stack_pointer = self.x;
    }

    #[allow(non_snake_case)]
    pub fn TYA(&mut self, _addressing_mode: AddressingMode) { // Transfer Y to Accumulator
        self.a = self.y;

        self.update_zero_and_negative_flags(self.y);
    }


    // STACK OPERATIONS
    #[allow(non_snake_case)]
    pub fn PHA(&mut self, _addressing_mode: AddressingMode) { // Push Accumulator
        self.stack_push(self.a);
    }

    #[allow(non_snake_case)]
    pub fn PHP(&mut self, _addressing_mode: AddressingMode) { // Push Processor Status
        let mut status = self.status.clone();
        status.break_command = true;
        status.unused = true;

        self.stack_push(status.to_byte());
    }

    #[allow(non_snake_case)]
    pub fn PLA(&mut self, _addressing_mode: AddressingMode) { // Pull Accumulator
        let value = self.stack_pop();
        self.a = value;
        
        self.update_zero_and_negative_flags(self.a);
    }

    #[allow(non_snake_case)]
    pub fn PLP(&mut self, _addressing_mode: AddressingMode) { // Pull Processor Status
        self.status = Status::from_byte(self.stack_pop());
        self.status.break_command = false;
        self.status.unused = true;
    }


    // LOGICAL OPERATIONS
    #[allow(non_snake_case)]
    pub fn AND(&mut self, addressing_mode: AddressingMode) { // Logical AND
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.a &= value;

        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.update_zero_and_negative_flags(self.a);
    }

    #[allow(non_snake_case)]
    pub fn EOR(&mut self, addressing_mode: AddressingMode) { // Exclusive OR
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.a ^= value;

        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.update_zero_and_negative_flags(self.a);
    }

    #[allow(non_snake_case)]
    pub fn ORA(&mut self, addressing_mode: AddressingMode) { // Logical OR
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        self.a |= value;

        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.update_zero_and_negative_flags(self.a);
    }


    // ARITHMETIC OPERATIONS
    #[allow(non_snake_case)]
    pub fn ADC(&mut self, addressing_mode: AddressingMode) { // Add with Carry
        let (value, page_boundary_cross) = self.get_data(addressing_mode);
        let addition = (self.a as u16)
            .wrapping_add(value as u16)
            .wrapping_add(self.status.carry as u16);
        let result = addition as u8;

        self.status.carry = addition > 0xFF;
        self.status.overflow = (value ^ result) & (result ^ self.a) & 0x80 != 0;
        
        self.indexed_cycles(addressing_mode, page_boundary_cross);
        
        self.a = result;
        self.update_zero_and_negative_flags(self.a);
    }

    #[allow(non_snake_case)]
    pub fn DEC(&mut self, addressing_mode: AddressingMode) { // Decrement Memory
        let address = self.get_address(addressing_mode).0;
        let value = self.read(address);
        let result = value.wrapping_sub(1);
        self.write(address, result);

        self.update_zero_and_negative_flags(result);
    }

    #[allow(non_snake_case)]
    pub fn DEX(&mut self, _addressing_mode: AddressingMode) { // Decrement X Register
        self.x = self.x.wrapping_sub(1);

        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn DEY(&mut self, _addressing_mode: AddressingMode) { // Decrement Y Register
        self.y = self.y.wrapping_sub(1);

        self.update_zero_and_negative_flags(self.y);
    }

    #[allow(non_snake_case)]
    pub fn INC(&mut self, addressing_mode: AddressingMode) { // Increment Memory
        let address = self.get_address(addressing_mode).0;
        let value = self.read(address);
        let result = value.wrapping_add(1);
        self.write(address, result);

        self.update_zero_and_negative_flags(result);
    }

    #[allow(non_snake_case)]
    pub fn INX(&mut self, _addressing_mode: AddressingMode) { // Increment X Register
        self.x = self.x.wrapping_add(1);

        self.update_zero_and_negative_flags(self.x);
    }

    #[allow(non_snake_case)]
    pub fn INY(&mut self, _addressing_mode: AddressingMode) { // Increment Y Register
        self.y = self.y.wrapping_add(1);

        self.update_zero_and_negative_flags(self.y);
    }

    #[allow(non_snake_case)]
    pub fn SBC(&mut self, addressing_mode: AddressingMode) { // Subtract with Carry
        let (mut value, page_boundary_cross) = self.get_data(addressing_mode);

        value = value.wrapping_neg();

        
        let addition = (self.a as u16)
            .wrapping_add(value as u16)
            .wrapping_sub(!self.status.carry as u16);
        let result = addition as u8;

        self.status.carry = addition > 0xFF;
        self.status.overflow = (value ^ result) & (result ^ self.a) & 0x80 != 0;
        
        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.a = result;
        self.update_zero_and_negative_flags(self.a);
    }


    // BIT OPERATIONS
    #[allow(non_snake_case)]
    pub fn ASL(&mut self, addressing_mode: AddressingMode) { // Arithmetic Shift Left
        if addressing_mode == AddressingMode::Accumulator {
            self.status.carry = (self.a & 0x80) != 0;
            self.a <<= 1;

            self.update_zero_and_negative_flags(self.a);
        } else {
            let address = self.get_address(addressing_mode).0;
            let value = self.read(address);

            self.status.carry = (value & 0x80) != 0;
            self.write(address, value << 1);

            self.update_zero_and_negative_flags(value << 1);
        }

    }

    #[allow(non_snake_case)]
    pub fn LSR(&mut self, addressing_mode: AddressingMode) {  // Logical Shift Right
        if addressing_mode == AddressingMode::Accumulator {
            self.status.carry = (self.a & 0x01) != 0;
            self.a >>= 1;

            self.update_zero_and_negative_flags(self.a);
        } else {
            let address = self.get_address(addressing_mode).0;
            let value = self.read(address);

            self.status.carry = (value & 0x01) != 0;
            let result = value >> 1;
            self.write(address, result);

            self.update_zero_and_negative_flags(result);
        }
    }

    #[allow(non_snake_case)]
    pub fn ROL(&mut self, addressing_mode: AddressingMode) { // Rotate Left
        if addressing_mode == AddressingMode::Accumulator {
            let carry = self.status.carry as u8;
            self.status.carry = (self.a & 0x80) != 0;
            self.a = (self.a << 1) | carry;

            self.update_zero_and_negative_flags(self.a);
        } else {
            let address = self.get_address(addressing_mode).0;
            let value = self.read(address);

            let carry = self.status.carry as u8;
            self.status.carry = (value & 0x80) != 0;
            let result = (value << 1) | carry;
            self.write(address, result);

            self.update_zero_and_negative_flags(result);
        }
    }

    #[allow(non_snake_case)]
    pub fn ROR(&mut self, addressing_mode: AddressingMode) { // Rotate Right
        if addressing_mode == AddressingMode::Accumulator {
            let carry = self.status.carry as u8;
            self.status.carry = (self.a & 0x01) != 0;
            self.a = (self.a >> 1) | (carry << 7);

            self.update_zero_and_negative_flags(self.a);
        } else {
            let address = self.get_address(addressing_mode).0;
            let value = self.read(address);

            let carry = self.status.carry as u8;
            self.status.carry = (value & 0x01) != 0;
            let result = (value >> 1) | (carry << 7);
            self.write(address, result);

            self.update_zero_and_negative_flags(result);
        }
    }

    
    // SUBROUTINE OPERATIONS
    #[allow(non_snake_case)]
    pub fn JMP(&mut self, addressing_mode: AddressingMode) { // Jump
        let address = self.get_address(addressing_mode).0;

        self.program_counter = address;
    }

    #[allow(non_snake_case)]
    pub fn JSR(&mut self, addressing_mode: AddressingMode) { // Jump to Subroutine
        let address = self.get_address(addressing_mode).0;

        self.program_counter += 1;

        self.stack_push((self.program_counter >> 8) as u8);
        self.stack_push((self.program_counter & 0xFF) as u8);

        self.program_counter = address;
    }

    #[allow(non_snake_case)]
    pub fn RTI(&mut self, _addressing_mode: AddressingMode) { // Return from Interrupt
        self.status = Status::from_byte(self.stack_pop());
        self.status.break_command = false;
        self.status.unused = true;
        
        let low = self.stack_pop();
        let high = self.stack_pop();
        self.program_counter = self.hilo_to_u16(high, low);
    }

    #[allow(non_snake_case)]
    pub fn RTS(&mut self, _addressing_mode: AddressingMode) { // Return from Subroutine
        let low = self.stack_pop();
        let high = self.stack_pop();

        self.program_counter = self.hilo_to_u16(high, low)
            .wrapping_add(1);
    }

    // COMPARISON OPERATIONS
    #[allow(non_snake_case)]
    pub fn BIT(&mut self, addressing_mode: AddressingMode) { // Bit Test
        let address = self.get_address(addressing_mode).0;
        let value = self.read(address);
        let result = self.a & value;

        self.status.zero = result == 0;
        self.status.overflow = (value & 0x40) != 0;
        self.status.negative = (value & 0x80) != 0;
    }

    #[allow(non_snake_case)]
    pub fn CMP(&mut self, addressing_mode: AddressingMode) { // Compare
        let (value, page_boundary_cross) = self.get_data(addressing_mode);

        self.status.carry = self.a >= value;
        
        self.indexed_cycles(addressing_mode, page_boundary_cross);

        self.update_zero_and_negative_flags(self.a.wrapping_sub(value));
    }

    #[allow(non_snake_case)]
    pub fn CPX(&mut self, addressing_mode: AddressingMode) { // Compare X Register
        let value= self.get_data(addressing_mode).0;

        self.status.carry = self.x >= value;
    
        self.update_zero_and_negative_flags(self.x.wrapping_sub(value));
    }

    #[allow(non_snake_case)]
    pub fn CPY(&mut self, addressing_mode: AddressingMode) { // Compare Y Register
        let value= self.get_data(addressing_mode).0;

        self.status.carry = self.y >= value;
    
        self.update_zero_and_negative_flags(self.y.wrapping_sub(value));
    }


    // BRANCHING OPERATIONS
    #[allow(non_snake_case)]
    pub fn BCC(&mut self, addressing_mode: AddressingMode) { // Branch if Carry Clear
        self.branch(addressing_mode, !self.status.carry);
    }

    #[allow(non_snake_case)]
    pub fn BCS(&mut self, addressing_mode: AddressingMode) { // Branch if Carry Set
        self.branch(addressing_mode, self.status.carry);
    }

    #[allow(non_snake_case)]
    pub fn BEQ(&mut self, addressing_mode: AddressingMode) { // Branch if Equa
        self.branch(addressing_mode, self.status.zero);
    }

    #[allow(non_snake_case)]
    pub fn BMI(&mut self, addressing_mode: AddressingMode) { // Branch if Minus
        self.branch(addressing_mode, self.status.negative);
    }

    #[allow(non_snake_case)]
    pub fn BNE(&mut self, addressing_mode: AddressingMode) { // Branch if Not Equal
        self.branch(addressing_mode, !self.status.zero);
    }

    #[allow(non_snake_case)]
    pub fn BPL(&mut self, addressing_mode: AddressingMode) { // Branch if Positive
        self.branch(addressing_mode, !self.status.negative);
    }

    #[allow(non_snake_case)]
    pub fn BVC(&mut self, addressing_mode: AddressingMode) { // Branch if Overflow Clear
        self.branch(addressing_mode, !self.status.overflow);
    }
    
    #[allow(non_snake_case)]
    pub fn BVS(&mut self, addressing_mode: AddressingMode) {  // Branch if Overflow Set
        self.branch(addressing_mode, self.status.overflow);
    }


    // STATUS & SYSTEM OPERATIONS
    #[allow(non_snake_case)]
    pub fn BRK(&mut self, _addressing_mode: AddressingMode) { // Force Interrupt
        self.program_counter = self.program_counter.wrapping_add(1);

        self.stack_push((self.program_counter >> 8) as u8);
        self.stack_push(self.program_counter as u8);

        self.status.break_command = true;
        self.stack_push(self.status.to_byte());
        self.status.break_command = false;

        let low = self.read(0xFFFE);
        let high = self.read(0xFFFF);
        self.program_counter = self.hilo_to_u16(high, low);

        self.complete = true;
    }
    
    #[allow(non_snake_case)]
    pub fn CLC(&mut self, _addressing_mode: AddressingMode) { // Clear Carry Flag
        self.status.carry = false;
    }
    
    #[allow(non_snake_case)]
    pub fn CLD(&mut self, _addressing_mode: AddressingMode) { // Clear Decimal Mode
        self.status.decimal = false;
    }
    
    #[allow(non_snake_case)]
    pub fn CLI(&mut self, _addressing_mode: AddressingMode) { // Clear Interrupt Disable
        self.status.interrupt = false;
    }
    
    #[allow(non_snake_case)]
    pub fn CLV(&mut self, _addressing_mode: AddressingMode) { // Clear Overflow Flag
        self.status.overflow = false;
    }
    
    #[allow(non_snake_case)]
    pub fn NOP(&mut self, _addressing_mode: AddressingMode) { // No Operation
        // nothing todo
    }
    
    #[allow(non_snake_case)]
    pub fn SEC(&mut self, _addressing_mode: AddressingMode) { // Set Carry Flag
        self.status.carry = true;
    }
    
    #[allow(non_snake_case)]
    pub fn SED(&mut self, _addressing_mode: AddressingMode) { // Set Decimal Mode
        self.status.decimal = true;
    }
    
    #[allow(non_snake_case)]
    pub fn SEI(&mut self, _addressing_mode: AddressingMode) { // Set Interrupt Disable
        self.status.interrupt = true;
    }
    
    
    // ilLeGaL OPERATIONS
    #[allow(non_snake_case)]
    pub fn ILLEGAL(&mut self, _addressing_mode: AddressingMode) { // Illegal Instruction
        todo!();
    }
}