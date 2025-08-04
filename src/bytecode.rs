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

fn opcode_to_enum(opcode: u16) -> Result<Opcode, InvalidInstruction> {
    match opcode {
        0b00000 => Ok(Opcode::Add),
        0b00001 => Ok(Opcode::Sub),
        0b00010 => Ok(Opcode::Or),
        0b00011 => Ok(Opcode::And),
        0b00100 => Ok(Opcode::Xor),
        0b00101 => Ok(Opcode::Not),
        0b00110 => Ok(Opcode::Shl),
        0b00111 => Ok(Opcode::Shr),
        0b10000 => Ok(Opcode::Lw),
        0b11000 => Ok(Opcode::Sw),
        0b11100 => Ok(Opcode::Branch),
        0b11101 => Ok(Opcode::Jump),
        0b11111 => Ok(Opcode::JumpReg),
        0b10100 => Ok(Opcode::Li),
        _ => Err(InvalidInstruction(format!("Unknown opcode: {:#06x}", opcode)))
    }
}

impl std::fmt::Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Or => "or",
            Opcode::And => "and",
            Opcode::Xor => "xor",
            Opcode::Not => "not",
            Opcode::Shl => "shl",
            Opcode::Shr => "shr",
            Opcode::Lw => "lw",
            Opcode::Sw => "sw",
            Opcode::Branch => "branch",
            Opcode::Jump => "jmp(i)",
            Opcode::JumpReg => "jmp(r)",
            Opcode::Li => "li"
        };
        write!(f, "{}", name)
    }
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

    ((opcode as u16) << 11) | ((rs as u16) << 8) | ((rt as u16) << 4) | rd as u16
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

pub fn disassemble(instr: u16) {
    let opcode = (instr >> 11) & 0b11111;
    let rs = (instr >> 8) & 0b111;
    let rt = (instr >> 4) & 0b1111;
    let rd = instr & 0b1111;
    let mem_off = (instr >> 8) & 0b111;
    let off = instr & 0b111111111;
    let imm = instr & 0b11111111;
    let cond = (instr >> 9) & 0b11;
    let opcode = opcode_to_enum(opcode).unwrap();

    match opcode {
        Opcode::Add | Opcode::Sub | Opcode::Or | Opcode::And | Opcode::Xor | Opcode::Not | Opcode::Shl | Opcode::Shr => {
            println!("{} r{}, r{}, r{}", opcode, rd, rt, rs);
        },
        Opcode::Lw | Opcode::Sw => {
            println!("{} r{}, r{}, {}", opcode, rd, rt, mem_off);
        },
        Opcode::Branch => {
            println!("{} {} r{}, {}", cond, opcode, rt, off);
        },
        Opcode::Jump => {
            println!("{} {}", opcode, off);
        },
        Opcode::JumpReg => {
            println!("jmp r{}", rd);
        },
        Opcode::Li => {
            println!("{} r{}, {}", opcode, rs, imm);
        }
    }
}
