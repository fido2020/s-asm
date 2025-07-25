use nom::{
    branch::alt, bytes::complete::tag, character::complete::{alpha1, alphanumeric0, space0}, combinator::{opt, recognize}, multi::{many0_count, separated_list0, separated_list1}, sequence::{delimited, pair}, Parser
};

enum Segment {
    Text,
    Data
}

enum Operand {
    Register(u8),
    Immediate(i32),
    Label(String),
}

struct Label(String, Segment, usize);
struct Instruction(String, Vec<Operand>);
struct Directive(String, Vec<Operand>);

struct AsmFile {
    instructions: Vec<Instruction>,
    labels: Vec<Label>,
}

enum AsmObject {
    Instruction(String, Vec<Operand>),
    Label(String),
    Directive(String, Vec<Operand>)
}

fn parse_name(input: &str) -> nom::IResult<&str, &str> {
    // Label names must start with _a-zA-Z and can contain _a-zA-Z0-9
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric0, tag("_"))))
    )).parse(input)
}

fn parse_label(input: &str) -> nom::IResult<&str, &str> {
    // Labels must start with _a-zA-Z and can contain _a-zA-Z0-9
    let (input, (label, colon)) = pair(
        parse_name,
        tag(":")
    ).parse(input)?;

    return Ok((input, label));
}

fn parse_operand(input: &str) -> nom::IResult<&str, Operand> {
    alt((
        // Register
        |i| {
            let (input, reg) = recognize(pair(tag("r"), nom::character::complete::digit1)).parse(i)?;
            let reg_num = reg[1..].parse::<u8>().unwrap();
            Ok((input, Operand::Register(reg_num)))
        },
        // Immediate
        |i| {
            let (input, imm) = nom::character::complete::i32(i)?;
            Ok((input, Operand::Immediate(imm)))
        },
        // Label
        |i| {
            let (input, label) = parse_name(i)?;
            Ok((input, Operand::Label(label.to_string())))
        }
    )).parse(input)
}

fn parse_operand_list(input: &str) -> nom::IResult<&str, Vec<Operand>> {
    // Parses a comma-separated list of operands
    separated_list0(delimited(space0, char(','), space0), parse_operand).parse(input)
}

fn parse_instruction(input: &str) -> nom::IResult<&str, Instruction> {
    let (input, (instr, operands)) = pair(
        parse_name,
        delimited(space0,  parse_operand_list, space0)
    ).parse(input)?;

    return Ok((input, Instruction(instr.to_string(), operands)))
}

fn parse_comment(input: &str) -> nom::IResult<&str, &str> {
    // Parses a comment starting with '#'
    let (input, _) = tag("#").parse(input)?;
    let (input, comment) = nom::character::complete::not_line_ending(input)?;
    Ok((input, comment))
}

fn parse_line(input: &str) -> nom::IResult<&str, AsmObject> {
    // Parsing logic goes here
    unimplemented!()
}

fn parse_asm(input: &str) -> nom::IResult<&str, AsmFile> {
    // Parsing logic goes here
    unimplemented!()
}
