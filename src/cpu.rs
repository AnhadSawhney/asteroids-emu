// emulate MOS 6502

use memory::Memory;

#[derive(Debug)]
enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    INVALID,
}

#[derive(Copy, Clone)]
enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageOffsetX,
    ZeroPageOffsetY,
    Absolute,
    AbsoluteOffsetX,
    AbsoluteOffsetY,
    Accumulator,
    OffsetXIndirect,
    IndirectOffsetY,
    IndirectLocation,
    AbsoluteLocation,
    NA,
}

impl AddressingMode {
    fn operand_string(&self, operand: Option<u16>) -> String {
        let op:u16 = match operand {
            None => 0,
            Some(n) => n,
        };
        match *self {
            AddressingMode::Immediate => {format!("#${:02X}", op)},
            AddressingMode::ZeroPage => {format!("${:02X}", op)},
            AddressingMode::ZeroPageOffsetX => {format!("${:02X},X", op)},
            AddressingMode::ZeroPageOffsetY => {format!("${:02X},Y", op)},
            AddressingMode::Absolute => {format!("${:04X}", op)},
            AddressingMode::AbsoluteOffsetX => {format!("${:04X},X", op)},
            AddressingMode::AbsoluteOffsetY => {format!("${:04X},Y", op)},
            AddressingMode::Accumulator => {format!("A")},
            AddressingMode::OffsetXIndirect => {format!("(${:02X},X)", op)},
            AddressingMode::IndirectOffsetY => {format!("(${:02X}),Y", op)},
            AddressingMode::IndirectLocation => {format!("(${:04X})", op)},
            AddressingMode::AbsoluteLocation => {format!("${:04X}", op)},
            AddressingMode::NA => {format!("")},
        }
    }
}

struct DecodedInstruction {
    address: u16,
    instruction: Instruction,
    addressing_mode: AddressingMode,
    operand: Option<u16>,
}

pub struct Cpu {
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    previous_pc: u16,   // for calculating branch cycles
    s: u8,
    p: u8,
    pub cycle: u64,
    debug_mode: bool,
}

impl Cpu {
    pub fn new(debug_mode: bool) -> Cpu {
        Cpu {a: 0, x: 0, y: 0, pc: 0, previous_pc: 0, s: 0, p: 0, cycle: 0, debug_mode}
    }

    fn get_word(addr: u16, memory: &Memory) -> u16 {
        (memory.get_byte(addr)) as u16 |
        ((memory.get_byte(addr + 1) as u16) << 8)
    }

    fn page_cross_penalty(addr: u16, offset: u8) -> u64 {
        if addr & 0xFF00 == (addr + offset as u16) & 0xFF00 {
            0
        }
        else {
            1
        }
    }

    pub fn reset(&mut self, memory: &Memory) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.p = 0b100;     // start with IRQ disable
        self.s = 0xFD;
        self.pc = Cpu::get_word(0xFFFC, memory);
        self.cycle = 6;
    }

    fn load_byte_from_pc(&mut self, memory: &Memory) -> u8 {
        let byte = memory.get_byte(self.pc);
        self.pc += 1;
        byte
    }

    fn load_word_from_pc(&mut self, memory: &Memory) -> u16 {
        self.load_byte_from_pc(memory) as u16 | 
        ((self.load_byte_from_pc(memory) as u16) << 8)
    }

    fn fetch_operand(&mut self, memory: &mut Memory,
                     addressing_mode: AddressingMode) -> Option<u16> {
        match addressing_mode {
            AddressingMode::Immediate |
            AddressingMode::ZeroPage |
            AddressingMode::ZeroPageOffsetX |
            AddressingMode::ZeroPageOffsetY |
            AddressingMode::OffsetXIndirect |
            AddressingMode::IndirectOffsetY =>
                Some(self.load_byte_from_pc(memory) as u16),
            AddressingMode::Absolute |
            AddressingMode::AbsoluteOffsetX |
            AddressingMode::AbsoluteOffsetY |
            AddressingMode::IndirectLocation |
            AddressingMode::AbsoluteLocation =>
                Some(self.load_word_from_pc(memory)),
            _ => None
        }
    }

    fn fetch_instruction(&mut self, memory: &mut Memory) -> DecodedInstruction {
        let address = self.pc;
        let op_code = self.load_byte_from_pc(memory);
        let bad_opcode = || {
            panic!("Invalid op code {:02X} encountered at address {:04X}. Processor hung.",
                   op_code, address)
        };
        // for now, any undocumented instructions will hang the processor
        match op_code {
            0x00 => DecodedInstruction {
                address,
                instruction: Instruction::BRK,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x20 => DecodedInstruction {
                address,
                instruction: Instruction::JSR,
                addressing_mode: AddressingMode::AbsoluteLocation,
                operand: Some(self.load_word_from_pc(memory))},
            0x40 => DecodedInstruction {
                address,
                instruction: Instruction::RTI,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x60 => DecodedInstruction {
                address,
                instruction: Instruction::RTS,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x08 => DecodedInstruction {
                address,
                instruction: Instruction::PHP,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x28 => DecodedInstruction {
                address,
                instruction: Instruction::PLP,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x48 => DecodedInstruction {
                address,
                instruction: Instruction::PHA,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x68 => DecodedInstruction {
                address,
                instruction: Instruction::PLA,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x88 => DecodedInstruction {
                address,
                instruction: Instruction::DEY,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xA8 => DecodedInstruction {
                address,
                instruction: Instruction::TAY,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xC8 => DecodedInstruction {
                address,
                instruction: Instruction::INY,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xE8 => DecodedInstruction {
                address,
                instruction: Instruction::INX,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x18 => DecodedInstruction {
                address,
                instruction: Instruction::CLC,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x38 => DecodedInstruction {
                address,
                instruction: Instruction::SEC,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x58 => DecodedInstruction {
                address,
                instruction: Instruction::CLI,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x78 => DecodedInstruction {
                address,
                instruction: Instruction::SEI,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x98 => DecodedInstruction {
                address,
                instruction: Instruction::TYA,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xB8 => DecodedInstruction {
                address,
                instruction: Instruction::CLV,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xD8 => DecodedInstruction {
                address,
                instruction: Instruction::CLD,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xF8 => DecodedInstruction {
                address,
                instruction: Instruction::SED,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x8A => DecodedInstruction {
                address,
                instruction: Instruction::TXA,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0x9A => DecodedInstruction {
                address,
                instruction: Instruction::TXS,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xAA => DecodedInstruction {
                address,
                instruction: Instruction::TAX,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xBA => DecodedInstruction {
                address,
                instruction: Instruction::TSX,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xCA => DecodedInstruction {
                address,
                instruction: Instruction::DEX,
                addressing_mode: AddressingMode::NA,
                operand: None},
            0xEA => DecodedInstruction {
                address,
                instruction: Instruction::NOP,
                addressing_mode: AddressingMode::NA,
                operand: None},
            op if op & 0b11111 == 0b10000 => {
                let instruction = match (op & 0b11100000) >> 5 {
                    0b000 => Instruction::BPL,
                    0b001 => Instruction::BMI,
                    0b010 => Instruction::BVC,
                    0b011 => Instruction::BVS,
                    0b100 => Instruction::BCC,
                    0b101 => Instruction::BCS,
                    0b110 => Instruction::BNE,
                    0b111 => Instruction::BEQ,
                    _ => Instruction::INVALID,
                };
                DecodedInstruction {
                    address,
                    instruction,
                    addressing_mode: AddressingMode::Immediate, // we cheat and use immediate for relative
                    operand: self.fetch_operand(memory, AddressingMode::Immediate)}
            }
            op if op & 0b11 == 0b01 => {
                let addressing_mode = match (op & 0b11100) >> 2 {
                    0b000 => AddressingMode::OffsetXIndirect,
                    0b001 => AddressingMode::ZeroPage,
                    0b010 => AddressingMode::Immediate,
                    0b011 => AddressingMode::Absolute,
                    0b100 => AddressingMode::IndirectOffsetY,
                    0b101 => AddressingMode::ZeroPageOffsetX,
                    0b110 => AddressingMode::AbsoluteOffsetY,
                    0b111 => AddressingMode::AbsoluteOffsetX,
                    _ => AddressingMode::NA,
                };
                let instruction = match (op & 0b11100000) >> 5 {
                    0b000 => Instruction::ORA,
                    0b001 => Instruction::AND,
                    0b010 => Instruction::EOR,
                    0b011 => Instruction::ADC,
                    0b100 => Instruction::STA,
                    0b101 => Instruction::LDA,
                    0b110 => Instruction::CMP,
                    0b111 => Instruction::SBC,
                    _ => Instruction::INVALID,
                };
                // the one bad combination here is STA in immediate mode
                if let Instruction::STA = instruction {
                    if let AddressingMode::Immediate = addressing_mode {
                        bad_opcode();
                    }
                }
                DecodedInstruction {
                    address,
                    instruction,
                    addressing_mode,
                    operand: self.fetch_operand(memory, addressing_mode)}
            },
            op if op & 0b11 == 0b10 => {
                let instruction = match (op & 0b11100000) >> 5 {
                    0b000 => Instruction::ASL,
                    0b001 => Instruction::ROL,
                    0b010 => Instruction::LSR,
                    0b011 => Instruction::ROR,
                    0b100 => Instruction::STX,
                    0b101 => Instruction::LDX,
                    0b110 => Instruction::DEC,
                    0b111 => Instruction::INC,
                    _ => Instruction::INVALID,
                };
                let addressing_mode = match (op & 0b11100) >> 2 {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::ZeroPage,
                    0b010 => AddressingMode::Accumulator,
                    0b011 => AddressingMode::Absolute,
                    0b101 => {
                        match instruction {
                            Instruction::LDX => AddressingMode::ZeroPageOffsetY,
                            Instruction::STX => AddressingMode::ZeroPageOffsetY,
                            _ => AddressingMode::ZeroPageOffsetX,
                        }
                    },
                    0b111 => {
                        match instruction {
                            Instruction::LDX => AddressingMode::AbsoluteOffsetY,
                            _ => AddressingMode::AbsoluteOffsetX,
                        }
                    }
                    _ => AddressingMode::NA,
                };
                // weed out instructions with incompatible addressing modes
                let bad_combination = match addressing_mode {
                    AddressingMode::NA => true,
                    AddressingMode::Immediate => 
                        match instruction {
                            Instruction::LDX => false,
                            _ => true,
                        },
                    AddressingMode::Accumulator =>
                        match instruction {
                            Instruction::STX => true,
                            Instruction::LDX => true,
                            Instruction::DEC => true,
                            Instruction::INC => true,
                            _ => false,
                        },
                    AddressingMode::AbsoluteOffsetX =>
                        match instruction {
                            Instruction::STX => true,
                            _ => false,
                        },
                    _ => false,
                };
                if bad_combination {
                    bad_opcode();
                }
                DecodedInstruction {
                    address,
                    instruction,
                    addressing_mode,
                    operand: self.fetch_operand(memory, addressing_mode)}
            },
            op if op & 0b11 == 0b00 => {
                let addressing_mode = match (op & 0b11100) >> 2 {
                    0b000 => AddressingMode::Immediate,
                    0b001 => AddressingMode::ZeroPage,
                    0b011 => {
                        if op & 0b11100000 == 0b01100000 {
                            AddressingMode::IndirectLocation    // JMP ()
                        }
                        else if op & 0b11100000 == 0b01000000 {
                            AddressingMode::AbsoluteLocation    // JMP
                        }
                        else {
                            AddressingMode::Absolute
                        }
                    },
                    0b101 => AddressingMode::ZeroPageOffsetX,
                    0b111 => AddressingMode::AbsoluteOffsetX,
                    _ => AddressingMode::NA,
                };
                let instruction = match (op & 0b11100000) >> 5 {
                    0b001 => Instruction::BIT,
                    0b010 | 0b011 => Instruction::JMP,
                    0b100 => Instruction::STY,
                    0b101 => Instruction::LDY,
                    0b110 => Instruction::CPY,
                    0b111 => Instruction::CPX,
                    _ => Instruction::INVALID,
                };
                if let Instruction::INVALID = instruction {
                    bad_opcode();
                }
                // weed out instructions with incompatible addressing modes
                let bad_combination = match addressing_mode {
                    AddressingMode::NA => true,
                    AddressingMode::Immediate => {
                        match instruction {
                            Instruction::LDY => false,
                            Instruction::CPY => false,
                            Instruction::CPX => false,
                            _ => true,
                        }
                    },
                    AddressingMode::ZeroPage => {
                        match instruction {
                            Instruction::JMP => true,
                            _ => false,
                        }
                    },
                    AddressingMode::ZeroPageOffsetX => {
                        match instruction {
                            Instruction::STY => false,
                            Instruction::LDY => false,
                            _ => true,
                        }
                    },
                    AddressingMode::AbsoluteOffsetX => {
                        match instruction {
                            Instruction::LDY => false,
                            _ => true,
                        }
                    },
                    _ => false,
                };
                if bad_combination {
                    bad_opcode();
                }
                DecodedInstruction {
                    address,
                    instruction,
                    addressing_mode,
                    operand: self.fetch_operand(memory, addressing_mode)}
            },
            _ => {bad_opcode()},
        }
    }

    fn show_processor_state(&self) {
        println!("A: {:02X}   X: {:02X}  Y: {:02X}  S: {:02X}  PC: {:04X}  P: {:08b} cycle: {}",
                 self.a, self.x, self.y, self.s, self.pc, self.p, self.cycle);
    }

    fn show_instruction(decoded_instruction: &DecodedInstruction) {
        print!("{:04X} {:?} {}",
               decoded_instruction.address,
               decoded_instruction.instruction,
               decoded_instruction.addressing_mode.operand_string(decoded_instruction.operand));
    }

    fn flag_set(&self, mask: u8) -> bool {
        (self.p & mask) == mask
    }

    fn carry_set(&self) -> bool {
        self.flag_set(0b1)
    }

    fn zero_set(&self) -> bool {
        self.flag_set(0b10)
    }

    fn decimal_set(&self) -> bool {
        self.flag_set(0b1000)
    }

    fn overflow_set(&self) -> bool {
        self.flag_set(0b1000000)
    }

    fn negative_set(&self) -> bool {
        self.flag_set(0b10000000)
    }

    fn update_flag(&mut self, set: bool, mask: u8) {
        if set {
            self.p = self.p | mask;
        }
        else {
            self.p = self.p & (! mask);
        }
    }

    fn update_carry(&mut self, set: bool) {
        self.update_flag(set, 0b1);
    }

    fn update_zero(&mut self, set: bool) {
        self.update_flag(set, 0b10);
    }

    fn update_irq_disable(&mut self, set: bool) {
        self.update_flag(set, 0b100);
    }

    fn update_decimal(&mut self, set: bool) {
        self.update_flag(set, 0b1000);
    }

    fn update_brk_command(&mut self, set: bool) {
        self.update_flag(set, 0b10000);
    }

    fn update_overflow(&mut self, set: bool) {
        self.update_flag(set, 0b1000000);
    }

    fn update_negative_from_byte(&mut self, byte: u8) {
        self.update_flag(byte & 0x80 == 0x80, 0b10000000);
    }

    fn realise_operand(&self, decoded_instruction: &DecodedInstruction, memory: &Memory) -> u16 {
        let op = if let Some(val) = decoded_instruction.operand {val} else {0};
        match decoded_instruction.addressing_mode {
            AddressingMode::Immediate => op,
            AddressingMode::ZeroPage => memory.get_byte(op) as u16,
            AddressingMode::ZeroPageOffsetX =>
                memory.get_byte((op + self.x as u16) & 0xFF) as u16,
            AddressingMode::ZeroPageOffsetY =>
                memory.get_byte((op + self.y as u16) & 0xFF) as u16,
            AddressingMode::Absolute => memory.get_byte(op) as u16,
            AddressingMode::AbsoluteOffsetX =>
                memory.get_byte(op + self.x as u16) as u16,
            AddressingMode::AbsoluteOffsetY =>
                memory.get_byte(op + self.y as u16) as u16,
            AddressingMode::Accumulator => self.a as u16,
            AddressingMode::OffsetXIndirect => {
                let addr = Cpu::get_word((op + self.x as u16) & 0xFF, memory);
                memory.get_byte(addr) as u16
            },
            AddressingMode::IndirectOffsetY => {
                let addr = Cpu::get_word(op, memory) + self.y as u16;
                memory.get_byte(addr) as u16
            },
            AddressingMode::IndirectLocation => Cpu::get_word(op, memory),
            AddressingMode::AbsoluteLocation => op,
            _ => 0,
        }
    }

    fn store_to_operand(&mut self, store: u8,
                        decoded_instruction: &DecodedInstruction,
                        memory: &mut Memory) {
        let op = if let Some(val) = decoded_instruction.operand {val} else {0};
        match decoded_instruction.addressing_mode {
            AddressingMode::ZeroPage => {
                memory.set_byte(op, store);
            },
            AddressingMode::ZeroPageOffsetX => {
                memory.set_byte((op + self.x as u16) & 0xFF, store);
            },
            AddressingMode::ZeroPageOffsetY => {
                memory.set_byte( (op + self.y as u16) & 0xFF, store);
            },
            AddressingMode::Absolute => {
                memory.set_byte(op, store);
            },
            AddressingMode::AbsoluteOffsetX => {
                memory.set_byte(op + self.x as u16, store);
            },
            AddressingMode::AbsoluteOffsetY => {
                memory.set_byte(op + self.y as u16, store);
            },
            AddressingMode::Accumulator => {
                self.a = store;
            },
            AddressingMode::OffsetXIndirect => {
                let addr = Cpu::get_word((op + self.x as u16) & 0xFF, memory);
                memory.set_byte(addr, store);
            },
            AddressingMode::IndirectOffsetY => {
                let addr = Cpu::get_word(op, memory) + self.y as u16;
                memory.set_byte(addr, store);
            },
            _ => {},
        }

    }

    fn branch(&mut self, op: u16) {
        if op & 0x80 == 0x80 {
            self.pc -= 0x100 - op;
        }
        else {
            self.pc += op;
        }
    }

    fn push_byte(&mut self, store: u8, memory: &mut Memory) {
        memory.set_byte(0x100 + self.s as u16, store);
        if self.s == 0 {
            self.s = 0xFF;
        }
        else {
            self.s -= 1;
        }
    }

    fn pop_byte(&mut self, memory: &Memory) -> u8 {
        if self.s == 0xFF {
            self.s = 0;
        }
        else {
            self.s += 1;
        }
        memory.get_byte(0x100 + self.s as u16)
    }

    fn push_word(&mut self, store: u16, memory: &mut Memory) {
        self.push_byte((store >> 8) as u8, memory);
        self.push_byte((store & 0xFF) as u8, memory);
    }

    fn pop_word(&mut self, memory: &mut Memory) -> u16 {
        self.pop_byte(memory) as u16 | ((self.pop_byte(memory) as u16) << 8)
    }

    fn decrement_byte(&mut self, byte: u8) -> u8 {
        let result = if byte == 0 {0xFF} else {byte - 1};
        self.update_zero(result == 0);
        self.update_negative_from_byte(result);
        result
    }

    fn increment_byte(&mut self, byte: u8) -> u8 {
        let result = if byte == 0xFF {0} else {byte + 1};
        self.update_zero(result == 0);
        self.update_negative_from_byte(result);
        result
    }

    fn compare(&mut self, compare_to: u8, byte: u8) {
        self.update_carry(compare_to >= byte);
        self.update_zero(compare_to == byte);
        self.update_negative_from_byte((compare_to as u16 + 0x100 - byte as u16) as u8);
    }

    pub fn initiate_nmi(&mut self, memory: &mut Memory) {
        let pc = self.pc;
        let p = self.p;
        self.push_word(pc, memory);
        self.push_byte(p, memory);
        self.pc = Cpu::get_word(0xFFFA, memory);
        self.cycle += 7;
    }

    pub fn execute_instruction(&mut self, memory: &mut Memory) {
        self.previous_pc = self.pc;
        if self.debug_mode {
            self.show_processor_state();
        }
        let decoded_instruction = self.fetch_instruction(memory);
        if self.debug_mode {
            Cpu::show_instruction(&decoded_instruction);
        }
        let op = self.realise_operand(&decoded_instruction, &memory);
        match decoded_instruction.instruction {
            Instruction::ADC => {
                let sum = if self.decimal_set() {
                    // not sure what should happen when digits are outside decimal range
                    let rhs = (self.a & 0xF) as u16 + (op & 0xF) +
                        if self.carry_set() {1} else {0};
                    let carry = if rhs > 9 {1} else {0};
                    let lhs = ((self.a & 0xF0) >> 4) as u16 +((op & 0xF0) >> 4) + carry;
                    self.update_carry(lhs > 9);
                    ((lhs % 10) << 4) | (rhs % 10)
                }
                else {
                    let result = self.a as u16 + op +
                        if self.carry_set() {1} else {0};
                    self.update_carry(result > 0xFF);
                    result & 0xFF
                };
                // same rules seem to apply for negative and overflow for bcd
                self.update_negative_from_byte(sum as u8);
                self.update_zero(sum == 0);
                let overflow = (self.a < 128 && sum > 127) ||
                    (self.a > 127 && sum < 128);
                self.update_overflow(overflow);
                self.a = sum as u8;
            }
            Instruction::AND => {
                let result = self.a & op as u8;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::ASL => {
                let result = op << 1;
                self.update_carry(result & 0x100 == 0x100);
                self.update_negative_from_byte(result as u8);
                self.update_zero(result & 0xFF == 0);
                self.store_to_operand((result & 0xFF) as u8,
                    &decoded_instruction, memory);
            },
            Instruction::BCC => {
                if ! self.carry_set() {
                    self.branch(op);
                }
            },
            Instruction::BCS => {
                if self.carry_set() {
                    self.branch(op);
                }
            },
            Instruction::BEQ => {
                if self.zero_set() {
                    self.branch(op);
                }
            },
            Instruction::BIT => {
                let result = self.a & op as u8;
                self.update_zero(result == 0);
                self.update_overflow(op & 0x40 == 0x40);
                self.update_negative_from_byte(op as u8);
            },
            Instruction::BMI => {
                if self.negative_set() {
                    self.branch(op);
                }
            },
            Instruction::BNE => {
                if ! self.zero_set() {
                    self.branch(op);
                }
            },
            Instruction::BPL => {
                if ! self.negative_set() {
                    self.branch(op);
                }
            },
            Instruction::BRK => {
                let ret_addr = self.pc;
                let flags = self.p;
                self.push_word(ret_addr, memory);
                self.push_byte(flags, memory);
                self.pc = Cpu::get_word(0xFFFE, memory);
                self.update_brk_command(true);
            },
            Instruction::BVC => {
                if ! self.overflow_set() {
                    self.branch(op);
                }
            },
            Instruction::BVS => {
                if self.overflow_set() {
                    self.branch(op);
                }
            },
            Instruction::CLC => {
                self.update_carry(false);
            },
            Instruction::CLD => {
                self.update_decimal(false);
            },
            Instruction::CLI => {
                self.update_irq_disable(false);
            },
            Instruction::CLV => {
                self.update_overflow(false);
            },
            Instruction::CMP => {
                let a = self.a;
                self.compare(a, op as u8);
            },
            Instruction::CPX => {
                let x = self.x;
                self.compare(x, op as u8);
            },
            Instruction::CPY => {
                let y = self.y;
                self.compare(y, op as u8);
            },
            Instruction::DEC => {
                let result = self.decrement_byte(op as u8);
                self.store_to_operand(result as u8, &decoded_instruction, memory);
            },
            Instruction::DEX => {
                let x = self.x;
                self.x = self.decrement_byte(x);
            },
            Instruction::DEY => {
                let y = self.y;
                self.y = self.decrement_byte(y);
            },
            Instruction::EOR => {
                let result = self.a ^ op as u8;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::INC => {
                let result = self.increment_byte(op as u8);
                self.store_to_operand(result as u8, &decoded_instruction, memory);
            },
            Instruction::INX => {
                let x = self.x;
                self.x = self.increment_byte(x);
            },
            Instruction::INY => {
                let y = self.y;
                self.y = self.increment_byte(y);
            },
            Instruction::JMP => {
                self.pc = op;
            },
            Instruction::JSR => {
                let ret_addr = self.pc - 1;
                self.push_word(ret_addr, memory);
                self.pc = op;
            },
            Instruction::LDA => {
                self.update_zero(op == 0);
                self.update_negative_from_byte(op as u8);
                self.a = op as u8;
            },
            Instruction::LDX => {
                self.update_zero(op == 0);
                self.update_negative_from_byte(op as u8);
                self.x = op as u8;
            },
            Instruction::LDY => {
                self.update_zero(op == 0);
                self.update_negative_from_byte(op as u8);
                self.y = op as u8;
            },
            Instruction::LSR => {
                self.update_carry(op & 0x1 == 0x1);
                let result = op >> 1;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result as u8);
                self.store_to_operand(result as u8, &decoded_instruction, memory);
            },
            Instruction::NOP => {},
            Instruction::ORA => {
                let result = self.a | op as u8;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::PHA => {
                let push = self.a;
                self.push_byte(push, memory);
            },
            Instruction::PHP => {
                let push = self.p;
                self.push_byte(push, memory);
            },
            Instruction::PLA => {
                let result = self.pop_byte(memory);
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::PLP => {
                self.p = self.pop_byte(memory);
            },
            Instruction::ROL => {
                let result = op << 1 | if self.carry_set() {1} else {0};
                self.update_carry(result & 0x100 == 0x100);
                self.update_zero(result & 0xFF == 0);
                self.update_negative_from_byte(result as u8);
                self.store_to_operand((result & 0xFF) as u8,
                    &decoded_instruction, memory);

            },
            Instruction::ROR => {
                let carry = op & 0x1 == 0x1;
                let result = op >> 1 | if self.carry_set() {0x80} else {0};
                self.update_carry(carry);
                self.update_zero(result == 0);
                self.update_negative_from_byte(result as u8);
                self.store_to_operand(result as u8, &decoded_instruction, memory);
            },
            Instruction::RTI => {
                self.p = self.pop_byte(memory);
                self.pc = self.pop_word(memory);
            },
            Instruction::RTS => {
                self.pc = self.pop_word(memory) + 1;
            },
            Instruction::SBC => {
                let sub = if self.decimal_set() {
                    // this is highly likely wrong...
                    // it is not exercised by the asteroids rom and who knows
                    // what actually happens when you subtract, say, CA from F3?
                    let lhs_a = (self.a & 0xF0) >> 4;
                    let lhs_o = (op as u8 & 0xF0) >> 4;
                    let rhs_a = self.a & 0xF;
                    let rhs_o = op as u8 & 0xF;
                    let borrow1 = if self.carry_set() {0} else {1};
                    let borrow2 = if rhs_a >= rhs_o + borrow1 {0} else {1};
                    let borrow3 = if lhs_a >= lhs_o + borrow2 {0} else {1};
                    let rhs = rhs_a + borrow2 * 10 - rhs_o - borrow1;
                    let lhs = lhs_a + borrow3 * 10 - lhs_o - borrow2;
                    self.update_carry(borrow3 == 0);
                    (lhs % 10) << 4 | (rhs % 10)
                }
                else {
                    let sub_tot = op + if self.carry_set() {0} else {1};
                    let new_carry = sub_tot <= self.a as u16;
                    self.update_carry(new_carry);
                    let result = self.a as u16 + 0x100 - sub_tot;
                    (result & 0xFF) as u8
                };
                // same rules seem to apply for negative and overflow for bcd
                self.update_negative_from_byte(sub as u8);
                self.update_zero(sub == 0);
                let overflow = (self.a < 128 && (sub & 0xFF) > 127) ||
                    (self.a > 127 && (sub & 0xFF) < 128);
                self.update_overflow(overflow);
                self.a = sub as u8;
            },
            Instruction::SEC => {
                self.update_carry(true);
            },
            Instruction::SED => {
                self.update_decimal(true);
            },
            Instruction::SEI => {
                self.update_irq_disable(true);
            },
            Instruction::STA => {
                let result = self.a;
                self.store_to_operand(result, &decoded_instruction, memory);
            },
            Instruction::STX => {
                let result = self.x;
                self.store_to_operand(result, &decoded_instruction, memory);
            },
            Instruction::STY => {
                let result = self.y;
                self.store_to_operand(result, &decoded_instruction, memory);
            },
            Instruction::TAX => {
                let result = self.a;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.x = result;
            },
            Instruction::TAY => {
                let result = self.a;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.y = result;
            },
            Instruction::TSX => {
                let result = self.s;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.x = result;
            },
            Instruction::TXA => {
                let result = self.x;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::TXS => {
                self.s = self.x;
            },
            Instruction::TYA => {
                let result = self.y;
                self.update_zero(result == 0);
                self.update_negative_from_byte(result);
                self.a = result;
            },
            Instruction::INVALID => {},
        }
	let cycles = self.instruction_cycles(&decoded_instruction, memory);
        self.cycle += cycles;
        if self.debug_mode {
            println!(" ({} cycles)\n", cycles);
        }
    }

    fn instruction_cycles(&self, decoded_instruction: &DecodedInstruction,
            memory: &Memory) -> u64 {
        
        match decoded_instruction.instruction {
            Instruction::ADC | Instruction::AND | Instruction::BIT |
            Instruction::CMP | Instruction::CPX | Instruction::CPY |
            Instruction::EOR | Instruction::LDA | Instruction::LDX |
            Instruction::LDY | Instruction::ORA | Instruction::SBC => {
                match decoded_instruction.addressing_mode {
                    AddressingMode::Immediate => 2,
                    AddressingMode::ZeroPage => 3,
                    AddressingMode::ZeroPageOffsetX => 4,
                    AddressingMode::Absolute => 4,
                    AddressingMode::AbsoluteOffsetX => {
                        4 +
                        if let Some(a) = decoded_instruction.operand {
                            Cpu::page_cross_penalty(a, self.x)
                        }
                        else {
                            0
                        }
                    },
                    AddressingMode::AbsoluteOffsetY => {
                        4 +
                        if let Some(a) = decoded_instruction.operand {
                            Cpu::page_cross_penalty(a, self.y)
                        }
                        else {
                            0
                        }
                    },
                    AddressingMode::OffsetXIndirect => 6,
                    AddressingMode::IndirectOffsetY => {
                        5 +
                        if let Some(a) = decoded_instruction.operand {
                            let addr = Cpu::get_word(a, memory);
                            Cpu::page_cross_penalty(addr, self.y)
                        }
                        else {
                            0
                        }
                    }
                    _ => {0},
                }
            },
            Instruction::ASL | Instruction::DEC | Instruction::INC |
            Instruction::LSR | Instruction::ROL | Instruction::ROR => {
                match decoded_instruction.addressing_mode {
                    AddressingMode::Accumulator => 2,
                    AddressingMode::ZeroPage => 5,
                    AddressingMode::ZeroPageOffsetX => 6,
                    AddressingMode::Absolute => 6,
                    AddressingMode::AbsoluteOffsetX => 7,
                    _ => 0,
                }
            },
            Instruction::BCC | Instruction::BCS | Instruction::BEQ |
            Instruction::BMI | Instruction::BNE | Instruction::BPL |
            Instruction::BVC | Instruction::BVS => {
                2 + 
                if self.pc != self.previous_pc + 2 {
                    if self.pc & 0xFF00 == (self.previous_pc + 2) & 0xFF00 {
                        1
                    }
                    else {
                        2
                    }
                }
                else {
                    0
                }
            },
            Instruction::BRK => 7,
            Instruction::CLC | Instruction::CLD | Instruction::CLI |
            Instruction::CLV | Instruction::DEX | Instruction::DEY |
            Instruction::INX | Instruction::INY | Instruction::NOP |
            Instruction::SEC | Instruction::SED | Instruction::SEI |
            Instruction::TAX | Instruction::TAY | Instruction::TSX |
            Instruction::TXS | Instruction::TXA | Instruction::TYA => 2,
            Instruction::JMP => {
                match decoded_instruction.addressing_mode {
                    AddressingMode::AbsoluteLocation => 3,
                    AddressingMode::IndirectLocation => 5,
                    _ => 0,
                }
            },
            Instruction::JSR | Instruction::RTI | Instruction::RTS => 6,
            Instruction::PHA | Instruction::PHP => 3,
            Instruction::PLA | Instruction::PLP => 4,
            Instruction::STA | Instruction::STX | Instruction::STY => {
                match decoded_instruction.addressing_mode {
                    AddressingMode::ZeroPage => 3,
                    AddressingMode::ZeroPageOffsetX => 4,
                    AddressingMode::Absolute => 4,
		    AddressingMode::AbsoluteOffsetX => 5,
                    AddressingMode::AbsoluteOffsetY => 5,
                    AddressingMode::OffsetXIndirect => 6,
                    AddressingMode::IndirectOffsetY => 6,
                    _ => 0,
                }
            },
            Instruction::INVALID => 0,
        }
    }
}
