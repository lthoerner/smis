mod errors;

use std::fs::File;
use std::io::{ BufRead, BufReader, Seek };
use crate::utilities::*;
use crate::utilities::instruction::Instruction;
use crate::utilities::symbol_table::SymbolTable;
use crate::utilities::string_methods::SMISString;
use self::errors::assembler_error::*;


pub fn start_assembler(asm_file_name: &str, bin_file_name: &str) -> Result<(), FileHandlerError> {
    // Open the input and output file
    let asm_file = File::options().read(true).open(asm_file_name)?;
    let bin_file = File::options().write(true).create(true).open(bin_file_name)?;

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&asm_file)?;
    
    dbg!("Symbol table: {}", symbol_table);

    match assemble_instructions(&asm_file, &bin_file) {
        Ok(()) => (),
        Err(_) => panic!("Something went wrong.")
    };

    Ok(())
}

// Scans the input ASM file for labels, and adds them to the symbol table for use later
fn read_labels(asm_file: &File) -> Result<SymbolTable, FileHandlerError> {
    // Store the address of the instruction currently being scanned
    let mut current_address: u16 = 0x00;
    
    // Stores all labels found in the file
    let mut symbol_table = symbol_table::new();
    
    let scanner = BufReader::new(asm_file);

    for line in scanner.lines() {
        // Handle any errors for line reading
        let line = match line {
            Ok(text) => text,
            Err(_) => return Err(FileHandlerError::ErrorInvalidFileContents)
        };

        let line = line.as_str();

        // Add any labels to the symbol table
        if is_label(line) {
            // TODO: Add get_label_name() and refactor
            symbol_table.add_label(match line.trim().strip_suffix(':') {
                Some(name) => name,
                // This should never happen, as the above condition requires the line to end in ':'
                None => panic!("[INTERNAL ERROR] Label was missing suffix")
            }, current_address);
        }

        // Current address is incremented by 2 because all instructions
        // are 32 bits, but the memory values are only 16 bits
        if !is_blankline(line) && !is_comment(line) && !is_label(line) { current_address += 2; }
    }

    Ok(symbol_table)
}

pub fn assemble_instructions(asm_file: &File, bin_file: &File) -> Result<(), AssemblerError> {
    let mut scanner = BufReader::new(asm_file);
    match scanner.rewind() {
        Ok(()) => (),
        Err(_) => return Err(AssemblerError::from(FileHandlerError::ErrorInvalidFileContents))
    };

    let mut line_count: u16 = 0;

    for line in scanner.lines() {
        line_count += 1;

        let line = match line {
            Ok(text) => text,
            Err(_) => return Err(AssemblerError::from(FileHandlerError::ErrorInvalidFileContents))
        };

        let line = line.trim();

        if is_blankline(line) || is_comment(line) || is_label(line) { continue; }

        let opcode = parse_opcode(line)?;

        // Gets an Instruction with the necessary format and the given opcode
        let instruction = match opcode_resolver::get_instruction_format(opcode) {
            Some(instr) => instr,
            None => return Err(AssemblerError::from(MnemonicParseError::ErrorUnknownMnemonic))
        };

        print!("[{}] ", line_count);
        match instruction {
            Instruction::RFormat { .. } => println!("{} is an R-Format", line),
            Instruction::IFormat { .. } => println!("{} is an I-Format", line),
            Instruction::JFormat { .. } => println!("{} is a J-Format", line)
        }
    }

    Ok(())
}

// Takes the instruction, gets the mnemonic, and translates it into an opcode
fn parse_opcode(instruction: &str) -> Result<u8, MnemonicParseError> {
    let mnemonic = match instruction.get_word(0) {
        Some(mnem) => mnem,
        None => return Err(MnemonicParseError::ErrorMissingMnemonic)
    };

    match opcode_resolver::get_opcode(mnemonic) {
        Some(op) => Ok(op),
        None => Err(MnemonicParseError::ErrorUnknownMnemonic)
    }
}

// Gets the register number operand from a given instruction by pulling the
// given operand with get_word() and parsing it using parse_register()
fn get_register(instruction: &str, index: usize) -> Result<u8, RegisterParseError> {
    let unparsed_register = match instruction.get_word(index) {
        Some(reg) => reg,
        None => return Err(RegisterParseError::ErrorMissingRegisterOperand)
    };

    Ok(parse_register(unparsed_register)?)
}

// Parses a register number from a string to a u8
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
            std::num::IntErrorKind::PosOverflow => return Err(RegisterParseError::ErrorNumberOutOfBounds),
            _ => return Err(RegisterParseError::ErrorNonNumeric)
        }
    };

    // Make sure the register exists (0-15)
    match register_num > 15 {
        true => Err(RegisterParseError::ErrorNumberOutOfBounds),
        false => Ok(register_num)
    }
}

// Gets the immediate value operand from a given instruction by pulling the
// last operand with get_word() and parsing it using parse_immediate()
fn get_immediate(instruction: &str) -> Result<u16, ImmediateParseError> {
    let unparsed_immediate = match instruction.get_word(instruction.count_words()) {
        Some(imm) => imm,
        None => return Err(ImmediateParseError::ErrorMissingImmediateOperand)
    };

    Ok(parse_immediate(unparsed_immediate)?)
}

// Parses an immediate value from a string to a u16
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
            std::num::IntErrorKind::PosOverflow => Err(ImmediateParseError::ErrorNumberOutOfBounds),
            _ => Err(ImmediateParseError::ErrorNonNumeric)
        }
    }
}

// Checks whether a given string ends with a ':', denoting that it is a jump label
fn is_label(line: &str) -> bool {
    // TODO: Possibly add checking for extra ':' at the end
    !is_comment(line) && line.trim().chars().last() == Some(':')
}

// Checks whether a given string starts with a "//", denoting that it is a comment
fn is_comment(line: &str) -> bool {
    line.trim().starts_with("//")
}

// Checks whether a given string is only whitespace, denoting that it is a blank line
fn is_blankline(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace())
}