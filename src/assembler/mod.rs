// TODO: Turn these off when no longer needed
#![allow(unused_imports)]

mod errors;

use crate::utilities::string_methods::SMISString;
use self::errors::assembler_error::AssemblerError;
use self::errors::assembler_error::FileHandlerError;
use self::errors::assembler_error::ParseError;
use self::errors::assembler_error::ImmediateParseError;


pub fn assemble(asm_file_name: &str, bin_file_name: &str) {
    
}

fn get_immediate(instruction: &str, index: usize) -> Result<u16, ImmediateParseError> {
    let unparsed_immediate = instruction.get_word(index);

    Ok(parse_immediate(unparsed_immediate)?)
}

fn parse_immediate(immediate: &str) -> Result<u16, ImmediateParseError> {
    // Make sure the immediate begins with '#'
    let trimmed_immediate = match immediate.strip_prefix('#') {
        Some(trim) => trim,
        None => return Err(ImmediateParseError::ErrorInvalidPrefix)
    };

    // Make sure the value after the prefix is numerical, then return it
    match trimmed_immediate.parse::<u16>() {
        Ok(val) => Ok(val),
        Err(_) => Err(ImmediateParseError::ErrorNonNumeric)
    }
}