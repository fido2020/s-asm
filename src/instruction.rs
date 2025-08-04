use std::{error, fmt};

use crate::{bytecode::{Instruction, InvalidInstruction, Opcode, Condition}, parser::Operand};

#[derive(Debug, Clone)]
pub struct InvalidOperands(PseudoInstruction, String);

impl std::fmt::Display for InvalidOperands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid operand for '{:?}': '{}'.", self.0, self.1)
    }
}

impl error::Error for InvalidOperands{}

#[derive(Debug, Clone, Copy)]
pub enum PseudoInstruction {
    Nop,
    Add,
    Sub,
    Or,
    And,
    Xor,
    Not,
    Shl,
    Shr,
    Cmp,
    Lw,
    Sw,
    Lb,
    Sb,
    Beq,
    Bne,
    Blt,
    Bge,
    Jump,
    Li,
    Push,
    Pop
}

impl PseudoInstruction {
    pub fn length(&self) -> u32 {
        // Push and pop expand to a sub/add and a load/store
        
        match self {
            PseudoInstruction::Push | PseudoInstruction::Pop => 3,
            _ => 1
        }
    }

    pub fn to_opcode(&self) -> Opcode {
        match self {
            PseudoInstruction::Nop => Opcode::Add, // NOP is treated as ADD with no operation
            PseudoInstruction::Add => Opcode::Add,
            PseudoInstruction::Sub => Opcode::Sub,
            PseudoInstruction::Or => Opcode::Or,
            PseudoInstruction::And => Opcode::And,
            PseudoInstruction::Xor => Opcode::Xor,
            PseudoInstruction::Not => Opcode::Not,
            PseudoInstruction::Shl => Opcode::Shl,
            PseudoInstruction::Shr => Opcode::Shr,
            PseudoInstruction::Cmp => Opcode::Add,
            PseudoInstruction::Lw => Opcode::Lw,
            PseudoInstruction::Sw => Opcode::Sw,
            PseudoInstruction::Beq => Opcode::Branch, // Branch on equal
            PseudoInstruction::Bne => Opcode::Branch, // Branch on not equal
            PseudoInstruction::Blt => Opcode::Branch, // Branch on less than
            PseudoInstruction::Bge => Opcode::Branch, // Branch on greater than or equal
            PseudoInstruction::Jump => Opcode::Jump,
            PseudoInstruction::Li => Opcode::Li,
            _ => panic!("Pseudo instruction does not have an opcode: {:?}", self),
        }
    }

    pub fn branch_condition(&self) -> Option<Condition> {
        match self {
            PseudoInstruction::Beq => Some(Condition::Equal),
            PseudoInstruction::Bne => Some(Condition::NotEqual),
            PseudoInstruction::Blt => Some(Condition::LessThan),
            PseudoInstruction::Bge => Some(Condition::GreaterThanEqual),
            _ => None,
        }
    }
}

enum OperandType {
    Reg,
    RegLo,
    Imm(u8),
}

pub fn name_to_op(name: &str) -> Result<PseudoInstruction, InvalidInstruction> {
    match name {
        "nop" => Ok(PseudoInstruction::Nop),
        "add" => Ok(PseudoInstruction::Add),
        "sub" => Ok(PseudoInstruction::Sub),
        "or" => Ok(PseudoInstruction::Or),
        "and" => Ok(PseudoInstruction::And),
        "xor" => Ok(PseudoInstruction::Xor),
        "not" => Ok(PseudoInstruction::Not),
        "shl" => Ok(PseudoInstruction::Shl),
        "shr" => Ok(PseudoInstruction::Shr),
        "cmp" => Ok(PseudoInstruction::Cmp),
        "lw" => Ok(PseudoInstruction::Lw),
        "sw" => Ok(PseudoInstruction::Sw),
        "beq" => Ok(PseudoInstruction::Beq),
        "bne" => Ok(PseudoInstruction::Bne),
        "blt" => Ok(PseudoInstruction::Blt),
        "bge" => Ok(PseudoInstruction::Bge),
        "jmp" => Ok(PseudoInstruction::Jump),
        "li" => Ok(PseudoInstruction::Li),
        "push" => Ok(PseudoInstruction::Push),
        "pop" => Ok(PseudoInstruction::Pop),
        invalid => Err(InvalidInstruction(invalid.into()))
    }
}

fn do_convert_operands(opcode: PseudoInstruction, defn: &[OperandType], operands: &[Operand]) -> Result<Vec<u16>, InvalidOperands> {
    if defn.len() != operands.len() {
        return Err(
            InvalidOperands(opcode, format!("Expected {} operands", defn.len()))
        );
    }

    let mut result = Vec::<u16>::new();

    for (i, o) in operands.iter().enumerate() {
        let real_op: Result<u16, String> = match &defn[i] {
            OperandType::Reg =>
                match o {
                    Operand::Register(r) => Ok((*r) as u16),
                    invalid => Err(invalid.to_string())
                },
            OperandType::RegLo =>
                match o {
                    Operand::Register(r) => {
                        if *r < 8 {
                            Ok(*r as u16)
                        } else {
                            Err(format!("Rs must be one of r0-r7"))
                        }
                    },
                    invalid => Err(invalid.to_string())
                },
            OperandType::Imm(bits) =>
                match o {
                    Operand::Immediate(val) => {
                        let unsigned = *val as u32;
                        if unsigned >= (1 << bits) {
                            eprintln!("Warning: truncating {} to {}-bits", val, bits);
                        }

                        Ok((*val & (1 << bits - 1)) as u16)
                    }
                    invalid => Err(invalid.to_string())
                }
        };

        let op = real_op
            .map_err(|e| InvalidOperands(opcode, e))?;

        result.push(op);
    }

    Ok(result.into())
}

pub fn make_single_insn(op: PseudoInstruction, operands: &[Operand]) -> Result<Instruction, InvalidOperands> {
    match op {
        PseudoInstruction::Nop => Ok(Instruction::Alu(Opcode::Add, 0, 0, 0)),
        PseudoInstruction::Add | PseudoInstruction::Sub | PseudoInstruction::And | PseudoInstruction::Or | PseudoInstruction::Xor
                | PseudoInstruction::Shl | PseudoInstruction::Shr => {
            let [p1, p2, p3] = do_convert_operands(op, &[OperandType::Reg, OperandType::Reg, OperandType::RegLo], operands)?[..]
                else { unreachable!() };

            Ok(Instruction::Alu(op.to_opcode(), p1, p2, p3))
        },
        PseudoInstruction::Not => {
            let [p1, p2] = do_convert_operands(op, &[OperandType::Reg, OperandType::Reg], operands)?[..]
                else { unreachable!() };

            Ok(Instruction::Alu(Opcode::Not, p1, p2, 0))
        },
        PseudoInstruction::Cmp => {
            let [p1, p2] = do_convert_operands(op, &[OperandType::Reg, OperandType::RegLo], operands)?[..]
                else { unreachable!() };

            Ok(Instruction::Alu(Opcode::Sub, 0, p1, p2))
        },
        PseudoInstruction::Lw | PseudoInstruction::Sw => {
            let [p1, p2, p3] = do_convert_operands(op, &[OperandType::Reg, OperandType::Reg, OperandType::Imm(3)], operands)?[..]
                else { unreachable!() };

            Ok(Instruction::Mem(op.to_opcode(), p1, p2, p3))
        },
        PseudoInstruction::Beq | PseudoInstruction::Bne | PseudoInstruction::Bge | PseudoInstruction::Blt => {
            let cond = op.branch_condition().unwrap();
            let [off] = do_convert_operands(op, &[OperandType::Imm(9)], operands)?[..]
                else { unreachable!() };

            Ok(Instruction::Branch(cond, off))
        }
        PseudoInstruction::Jump =>
            // Attempt both jmp <imm> and jmp <reg>
            match do_convert_operands(op, &[OperandType::Imm(11)], operands) {
                Ok(op) =>
                    Ok(Instruction::Jump(op[0])),
                Err(_) =>
                    do_convert_operands(PseudoInstruction::Jump, &[OperandType::Reg], operands)
                        .map(|op| Instruction::JumpReg(op[0]))
            },
        PseudoInstruction::Li => {
            let [reg, val] = do_convert_operands(op, &[OperandType::RegLo, OperandType::Imm(8)], operands)?[..]
                else { unreachable!() };
            Ok(Instruction::Li(reg, val))
        },
        _ => unreachable!()
    }
}

pub fn make_insns(op: PseudoInstruction, operands: &[Operand]) -> Result<Vec<Instruction>, InvalidOperands> {
    match op {
        PseudoInstruction::Push => {
            // Push expands to a sub/add and a load/store
            let [reg] = do_convert_operands(op, &[OperandType::Reg], operands)?[..]
                else { unreachable!() };

            Ok(vec![
                Instruction::Li(1, 4), // li r1, 1
                Instruction::Alu(Opcode::Sub, 15, 15, 1), // Sub r15, r15, r1
                Instruction::Mem(Opcode::Sw, reg, 15, 0)
            ])
        },
        PseudoInstruction::Pop => {
            // Pop expands to a load/store and an add/sub
            let [reg] = do_convert_operands(op, &[OperandType::Reg], operands)?[..]
                else { unreachable!() };

            Ok(vec![
                Instruction::Mem(Opcode::Lw, reg, 15, 0),
                Instruction::Li(1, 4), // li r1, 4
                Instruction::Alu(Opcode::Add, 15, 15, 1) // Add r15, r15, r1
            ])
        }
        simple_op =>
            make_single_insn(simple_op, operands).map(|insn| vec![insn])
    }
}
