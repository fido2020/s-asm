mod parser;
mod bytecode;
mod instruction;

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Read, Write};

use crate::bytecode::{disassemble, encode_instruction, Condition, Instruction};
use crate::instruction::{make_insns, name_to_op, PseudoInstruction};
use crate::parser::{parse_operand, AsmObject, Operand};

struct IncompleteInstruction(PseudoInstruction, Vec<Operand>);

fn assemble(input: &str) -> Result<(), Box<dyn Error>> {
    let objects = parser::parse_asm(input)?;

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
    let mut pc = 0;
    for obj in objects {
        match obj {
            AsmObject::Instruction(name, operands) => {
                let op = name_to_op(name.as_str())?;
                pc += op.length();

                instructions.push(
                    IncompleteInstruction(op, operands.iter().map(
                        |op| {
                            if let Operand::Name(name) = op {
                                if let Some(value) = constants.get(name) {
                                    return parse_operand(value).unwrap().1;
                                }
                            }

                            op.to_owned()
                        }
                    ).collect())
                )
            },
            AsmObject::Label(name) => {
                labels.insert( name, pc);
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
                    let off = (idx as i32) - (*label_addr as i32) + 1;

                    let mut value = (*label_addr) as i32;
                    // If the op is PseudoInstruction::Branch, we need the offset
                    if instr.0.is_branch() {
                        value = off as i32;
                    }

                    Ok(Operand::Immediate(value))
                },
                op => Ok(op.clone())
            }) {

            if op.is_err() {
                return Err(op.unwrap_err().into());
            }

            operands.push(op.unwrap());
        }

        let mut real_insns = 
            make_insns(instr.0, operands.as_slice())?
            .iter()
            .map(|insn| encode_instruction(*insn))
            .collect::<Vec<u16>>();

        bytecode.append(&mut real_insns);
    }

    for insn in bytecode {
        io::stdout().write_all(&[
            (insn >> 8) as u8, (insn & 0xff) as u8
        ])?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // check for disasm cmd line flag
    if std::env::args().any(|arg| arg == "--disasm") {
        let bytes = io::stdin().bytes()
            .map(|b| b.unwrap())
            .collect::<Vec<u8>>();

        if bytes.len() % 2 != 0 {
            eprintln!("Error: Bytecode length is not a multiple of 2");
            return Err("Bytecode length is not a multiple of 2".into());
        }

        let mut pc = 0;
        for chunk in bytes.chunks(2) {
            let insn = u16::from_be_bytes([chunk[0], chunk[1]]);
            disassemble(insn);
        }

        return Ok(())
    }

    let input = io::read_to_string(io::stdin())?;
    assemble(&input)?;

    Ok(())
}
