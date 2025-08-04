mod parser;
mod bytecode;
mod instruction;

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};

use crate::bytecode::{encode_instruction, Condition, Instruction, Opcode};
use crate::instruction::{check_convert_operands, name_to_opcode};
use crate::parser::{parse_operand, AsmObject, Operand};

struct IncompleteInstruction(Opcode, Vec<Operand>);

fn main() -> Result<(), Box<dyn Error>> {
    let input = io::read_to_string(io::stdin())?;
    let as_str = input.as_str();

    let objects = parser::parse_asm(as_str)?;

    let mut constants = HashMap::<String, String>::new();
    
    // Do two passes incase a constant relies on another constant
    for _i in 1..2 {
        for obj in &objects {
            if let AsmObject::Constant(key, value) = obj {
                let mut value = value;
                while constants.contains_key(value) {
                    value = constants.get(value).unwrap();
                }

                constants.insert(key.into(), value.into());
            }
        }
    }

    let mut labels = HashMap::<String, u32>::new();
    let mut instructions = Vec::<IncompleteInstruction>::new();
    for obj in objects {
        match obj {
            AsmObject::Instruction(name, operands) =>
                instructions.push(
                    IncompleteInstruction(name_to_opcode(name.as_str())?.into(), operands.iter().map(
                        |op| {
                            if let Operand::Name(name) = op {
                                if let Some(value) = constants.get(name) {
                                    return parse_operand(value).unwrap().1;
                                }
                            }

                            op.to_owned()
                        }
                    ).collect())
                ),
            AsmObject::Label(name) => {
                labels.insert( name, instructions.len() as u32);
            },
            _ =>
                ()
        }
    }

    let mut bytecode = Vec::<u16>::new();

    for ( idx, instr) in instructions.iter().enumerate() {
        let mut operands: Vec<Operand> = Vec::<Operand>::new();

        for op in
            instr.1.iter().map(|op| match op {
                Operand::Name(name) => {
                    let label_addr = labels.get(name);

                    if label_addr.is_none() {
                        return Err(format!("Invalid label: {}", name));
                    }

                    let label_addr = label_addr.unwrap();

                    // Get the offset from this insn to the label,
                    // include the one instruction offset
                    Ok(Operand::Immediate((idx as i32) - (*label_addr as i32) + 1))
                },
                op => Ok(op.clone())
            }) {

            if op.is_err() {
                return Err(op.unwrap_err().into());
            }

            operands.push(op.unwrap());
        }

        let checked_ops = 
            check_convert_operands(instr.0, operands.as_slice())?;

        let opcode = instr.0;

        let to_encode = match opcode {
            Opcode::Add | Opcode::Sub | Opcode::Cmp | Opcode::And | Opcode::Or | Opcode::Xor
                    | Opcode::Not | Opcode::Shl | Opcode::Shr =>
                Instruction::Alu(opcode, checked_ops[0], checked_ops[1], checked_ops[2]),
            Opcode::Lw | Opcode::Sw | Opcode::Lb | Opcode::Sb =>
                Instruction::Mem(opcode, checked_ops[0], checked_ops[1], checked_ops[2]),
            Opcode::Branch =>
                Instruction::Branch(Condition::Equal, checked_ops[0]),
            Opcode::Jump =>
                Instruction::Jump(checked_ops[0]),
            Opcode::Li =>
                Instruction::Li(checked_ops[0], checked_ops[1])
        };

        let encoded = encode_instruction(to_encode);

        bytecode.push(encoded);
    }

    for insn in bytecode {
        io::stdout().write_all(&[
            (insn >> 8) as u8, (insn & 0xff) as u8
        ])?;
    }

    Ok(())
}
