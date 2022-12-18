use super::super::errors::*;
use super::super::utilities::string_methods::SMISString;
use super::opcode_resolver;
use anyhow::{Context, Result};

// Takes the instruction, gets the mnemonic, and translates it into an opcode
pub fn get_opcode_from_instruction_text(instruction: &str) -> Result<u8> {
    match instruction.get_word(0) {
        Some(mnemonic) => match opcode_resolver::get_opcode(mnemonic) {
            Some(opcode) => Ok(opcode),
            None => Err(MnemonicParseError::UnknownMnemonic)
                .context("Unknown or malformed mnemonic.")
                .context(format!("At: '{}'", mnemonic)),
        },
        None => Err(MnemonicParseError::InvalidIndex)
            .context("[INTERNAL ERROR] Invalid mnemonic index access."),
    }
}

// Gets the register number operand from a given instruction by pulling the
// given operand with get_word() and parsing it using parse_register()
pub fn get_register(instruction: &str, index: usize) -> Result<u8> {
    match instruction.get_word(index) {
        Some(unparsed_register) => {
            parse_register(unparsed_register).context(format!("At: '{}'", unparsed_register))
        }
        None => Err(RegisterParseError::InvalidIndex)
            .context("[INTERNAL ERROR] Invalid register index access."),
    }
}

// Parses a register number from a string to a u8
pub fn parse_register(register: &str) -> Result<u8> {
    // Test special cases
    match register {
        "RZR" => return Ok(0),
        "RSP" => return Ok(15),
        "RBP" => return Ok(14),
        "RLR" => return Ok(13),
        _ => (),
    }

    // Make sure the register begins with 'R'
    let trimmed_register = match register.strip_prefix('R') {
        Some(trim) => trim,
        None => return Err(RegisterParseError::InvalidPrefix).context("Invalid register prefix."),
    };

    // TODO: Different error message for out of u8 bounds
    // Make sure the value after the prefix is numerical and within u8 bounds
    let register_num = trimmed_register
        .parse::<u8>()
        .map_err(|_| RegisterParseError::NonNumeric)
        .context("Non-numeric register number.")?;

    // Make sure the register exists (0-15)
    match register_num > 15 {
        true => Err(RegisterParseError::InvalidNumber)
            .context("Register number out of bounds (0-15).")
            .context(format!("At: '{}'", register)),
        false => Ok(register_num),
    }
}

// Gets the immediate value operand from a given instruction by pulling the
// last operand with get_word() and parsing it using parse_immediate()
pub fn get_immediate(instruction: &str) -> Result<u16> {
    // TODO: There could be more words between other operands and the immediate operand
    // Gets the last word of the line and attempts to parse it into an immediate value
    match instruction.get_word(instruction.count_words() - 1) {
        Some(unparsed_immediate) => {
            parse_immediate(unparsed_immediate).context(format!("At: '{}'", unparsed_immediate))
        }
        None => Err(ImmediateParseError::InvalidIndex)
            .context("[INTERNAL ERROR] Invalid immediate index access."),
    }
}

// Parses an immediate value from a string to a u16
pub fn parse_immediate(immediate: &str) -> Result<u16> {
    // Make sure the immediate begins with '#'
    let trimmed_immediate = match immediate.strip_prefix('#') {
        Some(trim) => trim,
        None => {
            return Err(ImmediateParseError::InvalidPrefix).context("Invalid immediate prefix.")
        }
    };

    // Make sure the value after the prefix is numerical and within u16 bounds, then return it
    trimmed_immediate
        .parse::<u16>()
        .map_err(|_| ImmediateParseError::NonNumeric)
        .context("Non-numeric immediate value.")
}

// Checks whether a given string ends with a ':', denoting that it is a jump label
pub fn is_label(line: &str) -> bool {
    // Forbids labels from containing whitespace
    if line.count_words() > 1 {
        return false;
    }

    // TODO: Possibly add checking for extra ':' at the end
    !is_comment(line) && line.ends_with(':')
}

// Checks whether a given string starts with a "//", denoting that it is a comment
pub fn is_comment(line: &str) -> bool {
    line.trim().starts_with("//")
}

// Checks whether a given string is only whitespace, denoting that it is a blank line
pub fn is_blankline(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace())
}
