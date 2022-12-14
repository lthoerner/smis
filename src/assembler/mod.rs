// TODO: Turn these off when no longer needed
#![allow(unused_imports)]

mod errors;

use std::num::IntErrorKind;

use crate::utilities::string_methods::SMISString;
use self::errors::assembler_error::AssemblerError;
use self::errors::assembler_error::FileHandlerError;
use self::errors::assembler_error::ParseError;
use self::errors::assembler_error::ImmediateParseError;
use self::errors::assembler_error::RegisterParseError;


pub fn assemble(asm_file_name: &str, bin_file_name: &str) {
    
}

fn get_register(instruction: &str, index: usize) -> Result<u8, RegisterParseError> {
    let unparsed_register = instruction.get_word(index);

    Ok(parse_register(unparsed_register)?)
}

fn parse_register(register: &str) -> Result<u8, RegisterParseError> {
    // Test special cases
    match register {
        "RZR" => return Ok(0),
        "RSP" => return Ok(15),
        "RBP" => return Ok(14),
        "RLR" => return Ok(13),
        _ => ()
    }

    // Make sure the register begins with 'R'
    let trimmed_register = match register.strip_prefix('R') {
        Some(trim) => trim,
        None => return Err(RegisterParseError::ErrorInvalidPrefix)
    };

    // Make sure the value after the prefix is numerical and within u8 bounds
    let register_num = match trimmed_register.parse::<u8>() {
        Ok(val) => val,
        Err(err) => match err.kind() {
            IntErrorKind::PosOverflow => return Err(RegisterParseError::ErrorNumberOutOfBounds),
            _ => return Err(RegisterParseError::ErrorNonNumeric)
        }
    };

    // Make sure the register exists (0-15)
    match register_num > 15 {
        true => Err(RegisterParseError::ErrorNumberOutOfBounds),
        false => Ok(register_num)
    }
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

    // Make sure the value after the prefix is numerical and within u16 bounds, then return it
    match trimmed_immediate.parse::<u16>() {
        Ok(val) => Ok(val),
        Err(err) => match err.kind() {
            IntErrorKind::PosOverflow => Err(ImmediateParseError::ErrorNumberOutOfBounds),
            _ => Err(ImmediateParseError::ErrorNonNumeric)
        }
    }
}

// Checks whether a given string ends with a ':', denoting that it is a jump label
fn is_label(line: &str) -> bool {
    // TODO: Possibly add checking for extra ':' at the end
    line.chars().last().unwrap() == ':'
}

// Checks whether a given string is a blank line, a comment, or a label, in which case it will be skipped over by the assembler
fn is_blankline_comment_label(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace()) || line.starts_with("//") || is_label(line)
}