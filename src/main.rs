mod parser;

use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let input = io::read_to_string(io::stdin())?;
    let as_str = input.as_str();

    let _ = parser::parse_asm(as_str)?;

    Ok(())
}
