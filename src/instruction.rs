use std::{error, fmt};

use crate::{bytecode::{InvalidInstruction, Opcode}, parser::Operand};

#[derive(Debug, Clone)]
pub struct InvalidOperands(Opcode, String);

impl std::fmt::Display for InvalidOperands {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid operand for '{:?}': '{}'.", self.0, self.1)
    }
}

impl error::Error for InvalidOperands{}

enum OperandType {
    Reg,
    RegLo,
    Imm(u8),
}

pub fn name_to_opcode(name: &str) -> Result<Opcode, InvalidInstruction> {
    match name {
        "nop" => Ok(Opcode::Add),
        "add" => Ok(Opcode::Add),
        "sub" => Ok(Opcode::Sub),
        "or" => Ok(Opcode::Or),
        "and" => Ok(Opcode::And),
        "xor" => Ok(Opcode::Xor),
        "not" => Ok(Opcode::Not),
        "shl" => Ok(Opcode::Shl),
        "shr" => Ok(Opcode::Shr),
        "cmp" => Ok(Opcode::Cmp),
        "lw" => Ok(Opcode::Lw),
        "sw" => Ok(Opcode::Sw),
        "lb" => Ok(Opcode::Lb),
        "sb" => Ok(Opcode::Sb),
        "beq" => Ok(Opcode::Branch),
        "bne" => Ok(Opcode::Branch),
        "bge" => Ok(Opcode::Branch),
        "bgt" => Ok(Opcode::Branch),
        "jmp" => Ok(Opcode::Jump),
        "li" => Ok(Opcode::Li),
        invalid => Err(InvalidInstruction(invalid.into()))
    }
}

fn do_convert_operands(opcode: Opcode, defn: &[OperandType], operands: &[Operand]) -> Result<Vec<u16>, InvalidOperands> {
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

                        Ok((*val as u8) as u16)
                    }
                    invalid => Err(invalid.to_string())
                }
        };

        let op = real_op
            .map_err(|e| InvalidOperands(opcode, e))?;

        result.push(op);
    }

    Ok(result)
}

pub fn check_convert_operands(opcode: Opcode, operands: &[Operand]) -> Result<Vec<u16>, InvalidOperands> {
    match opcode {
        Opcode::Add | Opcode::Sub | Opcode::Cmp | Opcode::And | Opcode::Or | Opcode::Xor
                | Opcode::Not | Opcode::Shl | Opcode::Shr =>
            do_convert_operands(opcode, &[OperandType::Reg, OperandType::Reg, OperandType::RegLo], operands),
        Opcode::Lw | Opcode::Sw | Opcode::Lb | Opcode::Sb =>
            do_convert_operands(opcode, &[OperandType::Reg, OperandType::Reg, OperandType::Imm(3)], operands),
        Opcode::Branch =>
            do_convert_operands(opcode, &[OperandType::Imm(9)], operands),
        Opcode::Jump =>
            do_convert_operands(opcode, &[OperandType::Imm(11)], operands),
        Opcode::Li =>
            do_convert_operands(opcode, &[OperandType::RegLo, OperandType::Imm(8)], operands)
    }
}
