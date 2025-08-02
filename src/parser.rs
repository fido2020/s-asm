use nom::{
    branch::alt, bytes::complete::tag, character::complete::{alpha1, alphanumeric0, alphanumeric1, char, space0, space1}, combinator::{not, opt, recognize}, error::ParseError, multi::{many0, many0_count, separated_list0, separated_list1}, sequence::{delimited, pair}, Parser
};
use std::error::Error;

#[derive(Debug)]
enum Segment {
    Text,
    Data
}

#[derive(Debug)]
enum Operand {
    Register(u8),
    Immediate(i32),
    Label(String),
}

#[derive(Debug)]
struct Label(String, Segment, usize);

#[derive(Debug)]
struct Instruction(String, Vec<Operand>);
#[derive(Debug)]
struct Directive(String, Vec<Operand>);

#[derive(Debug)]
struct AsmFile {
    instructions: Vec<Instruction>,
    labels: Vec<Label>,
}

#[derive(Debug)]
enum AsmObject {
    Instruction(String, Vec<Operand>),
    Label(String),
    Directive(String, Vec<Operand>),
    Constant(String, String)
}

fn parse_comment(input: &str) -> nom::IResult<&str, &str> {
    // Parses a comment starting with '#'
    let (input, _) = tag("#").parse(input)?;
    let (input, comment) = nom::character::complete::not_line_ending(input)?;
    Ok((input, comment))
}

fn parse_line_ending_comment(input: &str) -> nom::IResult<&str, &str> {
    // Parse a line ending with an optional comment
    let (input, _) = space0(input)?;
    let (input, _) = opt(parse_comment).parse(input)?;
    let (input, _) = nom::character::complete::line_ending(input)?;
    Ok((input, ""))
}

fn parse_name<'a>(input: &'a str) -> nom::IResult<&'a str, &str> {
    // Label names must start with _a-zA-Z and can contain _a-zA-Z0-9
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))
    )).parse(input)
}

fn parse_label<'a>(input: &'a str) -> nom::IResult<&'a str, AsmObject> {
    // Labels must start with _a-zA-Z and can contain _a-zA-Z0-9
    let (input, (label, colon)) = pair(
        parse_name,
        tag(":")
    ).parse(input)?;

    return Ok((input, AsmObject::Label(label.to_string())));
}

fn parse_constant<'a>(input: &'a str) -> nom::IResult<&'a str, AsmObject> {
    let (input, name) = parse_name(input)?;
    let (input, _) = delimited(space0, tag("="), space0).parse(input)?;
    let (input, value) = delimited(space0, alphanumeric1, parse_line_ending_comment).parse(input)?;

    return Ok((input, AsmObject::Constant(name.into(), value.into())));
}

fn parse_operand<'a>(input: &'a str) -> nom::IResult<&'a str, Operand> {
    alt((
        // Register
        |i: &'a str| {
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

fn parse_instruction(input: &str) -> nom::IResult<&str, AsmObject> {
    let (input, (instr, operands)) = pair(
        parse_name,
        delimited(space0,  parse_operand_list, opt(parse_line_ending_comment))
    ).parse(input)?;

    return Ok((input, AsmObject::Instruction(instr.to_string(), operands)))
}

pub fn parse_comment_whitespace(input: &str) -> nom::IResult<&str, &str> {
    let (input, _) = many0(alt((space1, parse_line_ending_comment))).parse(input)?;

    return Ok((input, ""))
}

pub fn parse_asm<'a>(input: &'a str) -> Result<(), Box<dyn Error>> {
    let (input, objects) = many0(delimited(parse_comment_whitespace, alt((
        parse_constant,
        parse_label,
        parse_instruction,
    )), parse_comment_whitespace)).parse(input)
        .map_err(|e| "Error")?;

    println!("{:?}", objects);
    println!("{:?}", input);

    return Ok(());
}
