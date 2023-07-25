use crate::utilities::{
    errors::*,
    instructions::{Instruction, InstructionContainer},
    messages,
    opcodes::Opcode,
    symbol_table::{self, SymbolTable},
    SmisString,
};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, Write};

// Initiates the assembly of the given ASM text file into a binary machine code file
pub fn start_assembler(assembly_filename: &str, binary_filename: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !assembly_filename.ends_with(".txt") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Input file must have a .txt extension.")
            .context(messages::USAGE);
    }

    if !binary_filename.ends_with(".bin") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Output file must have a .bin extension.")
            .context(messages::USAGE);
    }

    // Open/create the input and output file
    let Ok(assembly_file) = File::options().read(true).open(assembly_filename) else {
        return Err(FileHandlerError::FileOpenFailed)
            .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.");
    };

    let Ok(mut binary_file) = File::options().write(true).create(true).open(binary_filename) else {
        return Err(FileHandlerError::FileCreateFailed)
            .context("Couldn't open or create the output file. Make sure the file is not write-protected if it already exists.");
    };

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&assembly_file)?;

    // Assemble all the instructions and catch any errors
    // Write the assembled instructions to the output file
    write_output(
        &mut binary_file,
        &assemble_instructions(&assembly_file, &symbol_table)?,
    )?;

    Ok(())
}

// Writes the assembled instructions to the output machine code file
fn write_output(binary_file: &mut File, assembled_instructions: &Vec<u32>) -> Result<()> {
    for &instruction in assembled_instructions {
        // Instruction is converted to big-endian (network byte order) before being written to the file
        binary_file
            .write_all(&instruction.to_be_bytes())
            .map_err(|_| FileHandlerError::FileWriteFailed)
            .context("[INTERNAL ERROR] Couldn't write instructions to the binary file.")?;
    }

    Ok(())
}

// Scans the input ASM file for labels, and adds them to the symbol table for use later
fn read_labels(assembly_file: &File) -> Result<SymbolTable> {
    // Stores all labels found in the file along with their corresponding instruction addressses
    let mut symbol_table = symbol_table::new();

    let mut reader = BufReader::new(assembly_file);
    reader
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for symbol table pass.")?;

    // Store the address of the instruction currently being scanned
    let mut current_address: u16 = 0x00;

    // For each line in the file
    for line in reader.lines() {
        // Handle any errors for line reading
        let line = line.map_err(|_| FileHandlerError::FileReadFailed).context(
            "[INTERNAL ERROR] Couldn't read a line from the ASM file for symbol table pass.",
        )?;

        let line = line.trim();

        // Add any labels to the symbol table
        if is_label(line) {
            // TODO: Add get_label_name() and refactor
            symbol_table.add_label(
                match line.strip_suffix(':') {
                    Some(name) => name,
                    // This should never happen, as the above condition requires the line to end in ':'
                    None => {
                        return Err(SymbolTableError::CouldNotAddLabel)
                            .context("[INTERNAL ERROR] Label was missing suffix.")
                    }
                },
                current_address,
            );
        }

        // Current address is incremented by 2 because all instructions
        // are 32 bits, but the memory values are only 16 bits
        if !is_blankline(line) && !is_comment(line) && !is_label(line) {
            current_address += 2;
        }
    }

    Ok(symbol_table)
}

// Reads the ASM file and returns a Vec of the assembled instructions
fn assemble_instructions(assembly_file: &File, symbol_table: &SymbolTable) -> Result<Vec<u32>> {
    let mut reader = BufReader::new(assembly_file);
    reader
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for assembler pass.")?;

    let mut assembled_instructions = Vec::<u32>::new();

    // Line count is stored to give more descriptive error messages
    let mut line_count: u16 = 0;

    // For each line in the file
    for line in reader.lines() {
        line_count += 1;

        // Handle any errors for line reading
        let line = line.map_err(|_| FileHandlerError::FileReadFailed).context(
            "[INTERNAL ERROR] Couldn't read a line from the ASM file for the assembler pass.",
        )?;

        let line = line.trim();

        // Skip non-instruction lines
        if is_blankline(line) || is_comment(line) || is_label(line) {
            continue;
        }

        // Encode and assemble the instruction, then add it to the Vec
        let assembled_instruction = InstructionContainer::assemble(line, symbol_table)
            .context(format!("On line: {}", line_count))?
            .encode();
        assembled_instructions.push(assembled_instruction);
    }

    Ok(assembled_instructions)
}

// Takes the instruction, gets the mnemonic, and translates it into an opcode
pub fn get_opcode_from_mnemonic(instruction: &str) -> Result<Opcode> {
    let mnemonic = instruction.get_word(0);

    if let Some(mnemonic) = mnemonic {
        return Opcode::try_from(mnemonic.to_owned()).context(format!("At: '{}'", mnemonic));
    }

    Err(MnemonicParseError::InvalidIndex).context("[INTERNAL ERROR] Invalid mnemonic index access.")
}

// Gets the register identifier operand from a given instruction
pub fn get_register(instruction: &str, index: usize) -> Result<u8> {
    match instruction.get_word(index) {
        Some(unparsed_register) => parse_register_identifier(unparsed_register)
            .context(format!("At: '{}'", unparsed_register)),
        None => Err(RegisterParseError::InvalidIndex)
            .context("[INTERNAL ERROR] Invalid register index access."),
    }
}

// Parses a register identifier from a string to a u8
pub fn parse_register_identifier(register: &str) -> Result<u8> {
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
        .context("Non-numeric register index.")?;

    // Make sure the register exists (0-15)
    if register_num > 15 {
        return Err(RegisterParseError::InvalidNumber)
            .context("Register index out of bounds (0-15).")
            .context(format!("At: '{}'", register));
    }

    Ok(register_num)
}

// Gets the immediate value operand from a given instruction by pulling the
// last operand with get_word() and parsing it using parse_immediate()
pub fn get_immediate(instruction: &str) -> Result<u16> {
    // TODO: There could be more words between other operands and the immediate operand
    // Gets the last word of the line and attempts to parse it into an immediate value
    match instruction.get_word(instruction.count_words() - 1) {
        Some(unparsed_immediate) => parse_immediate_value(unparsed_immediate)
            .context(format!("At: '{}'", unparsed_immediate)),
        None => Err(ImmediateParseError::InvalidIndex)
            .context("[INTERNAL ERROR] Invalid immediate index access."),
    }
}

// Parses an immediate value from a string to a u16
pub fn parse_immediate_value(immediate: &str) -> Result<u16> {
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
    // TODO: Add specific error message for this
    if line.count_words() > 1 {
        return false;
    }

    // Requires labels to end with a ':'
    let Some(line) = line.strip_suffix(':') else {
        return false;
    };

    // Prevents commented lines from being parsed as labels
    // Forbids labels from containing multiple ':'
    // If the line still contains a ':', the remaining ':' is not at the end of the line,
    // meaning that the label is invalid
    !is_comment(line) && !line.contains(':')
}

// Checks whether a given string starts with a "//", denoting that it is a comment
pub fn is_comment(line: &str) -> bool {
    line.trim().starts_with("//")
}

// Checks whether a given string is only whitespace, denoting that it is a blank line
pub fn is_blankline(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace())
}
