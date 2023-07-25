use crate::utilities::{
    errors::*,
    instructions::{Instruction, InstructionContainer},
    messages,
    opcodes::{self, EncodingFormat},
    symbol_table::SymbolTable,
};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read, Seek, Write};

// Initiates the disassembly of the given binary machine code file into an ASM text file
pub fn start_disassembler(binary_filename: &str, assembly_filename: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !binary_filename.ends_with(".bin") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Input file must have a .bin extension.")
            .context(messages::USAGE);
    }

    if !assembly_filename.ends_with(".txt") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Output file must have a .txt extension.")
            .context(messages::USAGE);
    }

    // Open/create the input and output file
    let Ok(binary_file) = File::options().read(true).open(binary_filename) else {
        return Err(FileHandlerError::FileOpenFailed)
            .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.");
    };

    let Ok(mut assembly_file) = File::options()
        .write(true)
        .create(true)
        .open(assembly_filename)
    else {
        return Err(FileHandlerError::FileOpenFailed)
            .context("Couldn't open or create the output file. Make sure the file is not write-protected if it already exists.");
    };

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&binary_file)?;

    // Disassemble all the instructions and catch any errors
    // Write the disassembled instructions to the output file
    write_output(
        &mut assembly_file,
        disassemble_instructions(&binary_file, &symbol_table)?,
    )?;

    Ok(())
}

// Writes the disassembled instructions to the output ASM text file
fn write_output(assembly_file: &mut File, disassembled_instructions: Vec<String>) -> Result<()> {
    for mut instruction in disassembled_instructions {
        instruction.push('\n');
        if assembly_file.write_all(instruction.as_bytes()).is_err() {
            return Err(FileHandlerError::FileWriteFailed)
                .context("[INTERNAL ERROR] Couldn't write instructions to the assembly file.");
        }
    }

    Ok(())
}

// Scans the input machine code file for labels, and adds them to the symbol table for use later
fn read_labels(binary_file: &File) -> Result<SymbolTable> {
    // Stores all labels found in the file
    let mut symbol_table = SymbolTable::default();

    let mut reader = BufReader::new(binary_file);
    reader
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the machine code file for symbol table pass.")?;

    // Store the current label number
    let mut current_label: u16 = 0;

    // Read each instruction from the file
    loop {
        // Stores the current instruction
        let mut buffer = [0; 4];

        // Read 4-byte chunks of the file (instructions)
        match reader.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => {
                    return Err(FileHandlerError::FileReadFailed)
                        .context("The provided machine code file is not evenly divisible by memory words, and therefore is invalid or corrupted.")
                }
            },
        }

        // Take the bytes and put them in a single u32, converting from network byte order if needed
        let encoded_instruction = u32::from_be_bytes(buffer);

        let Some(opcode) = opcodes::extract_opcode(encoded_instruction) else {
            return Err(OpcodeParseError::UnknownOpcode)
                .context("Encountered invalid opcode.")
                .context(format!("At: '0x{:08X}'", encoded_instruction));
        };

        // If the instruction is a J-Type and its label is unique, add it to the symbol table
        if EncodingFormat::from(opcode) == EncodingFormat::J {
            let label_address = extract_address(encoded_instruction);

            if !symbol_table.contains(label_address) {
                symbol_table.add_label(&generate_label_name(current_label), label_address)?;

                current_label += 1;
            }
        }
    }

    Ok(symbol_table)
}

// TODO: Split this function into smaller functions
// Reads the machine code file and returns a Vec of the disassembled instructions
fn disassemble_instructions(binary_file: &File, symbol_table: &SymbolTable) -> Result<Vec<String>> {
    let mut reader = BufReader::new(binary_file);
    reader
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the machine code file for disassembler pass.")?;

    // Current address is stored to determine if a label should be printed
    let mut current_address: u16 = 0x00;

    let mut disassembled_instructions = Vec::<String>::new();

    // Read each instruction from the file
    loop {
        // If the label exists in the symbol table, add it to the Vec
        if let Some(label) = symbol_table.find_name(current_address) {
            // If a label appears at the beginning of the file, leading line break is not added
            disassembled_instructions.push(match current_address {
                0x00 => format!("{}:\n", label),
                _ => format!("\n{}:\n", label),
            })
        }

        current_address += 2;

        // Stores the current instruction
        let mut buffer = [0; 4];

        // Read 4-byte chunks of the file (instructions)
        match reader.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(FileHandlerError::FileReadFailed).context(
                    "[INTERNAL ERROR] Couldn't read the machine code file for symbol table pass.",
                ),
            },
        }

        // Take the bytes and put them in a single u32, converting from network byte order if needed
        let encoded_instruction = u32::from_be_bytes(buffer);

        // Decode and disassemble the instruction, then add it to the Vec
        let disassembled_instruction = match InstructionContainer::decode(encoded_instruction) {
            Ok(instruction) => instruction.disassemble(symbol_table)?,
            Err(e) => return Err(e).context(format!("At: '0x{:04X}'", current_address)),
        };

        disassembled_instructions.push(disassembled_instruction);
    }

    Ok(disassembled_instructions)
}

// Gets a generic label name based on the given label number
pub fn generate_label_name(label_number: u16) -> String {
    format!("Label_{}", label_number)
}

// Formats a register index into a register identifier
pub fn format_register(register: u8) -> Result<String> {
    if register > 15 {
        return Err(RegisterParseError::InvalidIndex)
            .context("Register index out of bounds (0-15).")
            .context(format!("At: '{}'", register));
    }

    // Special cases
    match register {
        0 => return Ok("RZR".to_string()),
        15 => return Ok("RSP".to_string()),
        14 => return Ok("RBP".to_string()),
        13 => return Ok("RLR".to_string()),
        _ => (),
    }

    // Standard register format
    Ok(format!("R{}", register))
}

// Formats an immediate value into a string
pub fn format_immediate(immediate: u16) -> String {
    format!("#{}", immediate)
}

// Gets an indexed register operand from the instruction
// Assumes that the index is between 0-2 (inclusive), because using Result<u8>
// would lead to way more complexity with no real benefit
pub fn extract_register(instruction: u32, index: usize) -> u8 {
    // Grab the register from the instruction by masking out a 4-bit section (shifted by the index),
    // then shifting it back to the rightmost 4 bits
    ((instruction & (0x00F00000u32 >> (index * 4))) >> (20 - (index * 4))) as u8
}

// Gets the immediate operand from the instruction
pub fn extract_immediate(instruction: u32) -> u16 {
    (instruction & 0x0000FFFF) as u16
}

// Gets the label address from the instruction
// Functionally the same as extract_immediate() but included for clarity
pub fn extract_address(instruction: u32) -> u16 {
    extract_immediate(instruction)
}
