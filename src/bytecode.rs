use std::error;

#[derive(Debug, Clone)]
pub struct InvalidInstruction(pub String);

impl std::fmt::Display for InvalidInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid instruction: '{}'.", self.0)
    }
}

impl error::Error for InvalidInstruction{}

#[derive(Debug, Clone, Copy)]
pub enum Condition {
    Equal = 0b00,
    NotEqual = 0b01,
    LessThan = 0b10,
    GreaterThanEqual = 0b11
}

#[derive(Debug, Clone, Copy)]
pub enum Opcode {
    Add = 0b00000,
    Sub = 0b00001,
    Or  = 0b00010,
    And = 0b00011,
    Xor = 0b00100,
    Not = 0b00101,
    Shl = 0b00110,
    Shr = 0b00111,
    Lw = 0b10000,
    Sw = 0b11000,
    Branch = 0b11100,
    Jump = 0b11101,
    JumpReg = 0b11111,
    Li = 0b10100
    //Cmp = 0b01001,
}

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Nop,
    Alu(Opcode, u16, u16, u16),
    Mem(Opcode, u16, u16, u16),
    Branch(Condition, u16),
    Jump(u16),
    JumpReg(u16),
    Li(u16, u16)
}

fn encode_alu_instruction(opcode: Opcode, rd: u16, rt: u16, rs: u16) -> u16 {
    assert!(rs < 8);

    ((opcode as u16) << 11) | ((rs as u16) << 8) | ((rs as u16) << 4) | rd as u16
}

fn encode_mem_instruction(opcode: Opcode, rd: u16, rt: u16, off: u16) -> u16 {
    ((opcode as u16) << 11) | ((off as u16) << 8) | ((rt as u16) << 4) | rd as u16
}

pub fn encode_instruction(instr: Instruction) -> u16 {
    match instr {
        Instruction::Nop => 0,
        Instruction::Alu(opcode, rd, rt, rs) =>
            encode_alu_instruction(opcode, rd, rt, rs),
        Instruction::Mem(opcode, rd, rt, off) =>
            encode_mem_instruction(opcode, rd, rt, off),
        Instruction::Branch(cond, off) => {
            assert!(off <= 0b111111111);
            (Opcode::Branch as u16) | ((cond as u16) << 9) | (off as u16)
        },
        Instruction::Jump(off) => {
            assert!(off <= 0b11111111111);
            (Opcode::Jump as u16) | off
        },
        Instruction::Li(reg, val) => {
            // ooooo ddd IIIIIIII

            assert!(val <= 0b11111111);
            ((Opcode::Li as u16) << 11) | (reg << 8) | (val)
        },
        Instruction::JumpReg(reg) => {
            eprintln!("warn: jmp <reg> unimplemented");

            ((Opcode::JumpReg as u16) << 11) | reg
        }
    }
}
