use crate::errors::*;
use crate::utilities::*;
use crate::utilities::symbol_table::SymbolTable;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, Seek, Read, Write, ErrorKind};

// Initiates the disassembly of the given binary machine code file into an ASM text file
pub fn start_disassembler(bin_file_name: &str, asm_file_name: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !bin_file_name.ends_with(".bin") {
        return Err(FileHandlerError::ErrorInvalidExtension)
            .context("Input file must have a .bin extension.")
            .context(user_messages::USAGE_ERROR);
    }

    // Open/create the input and output file
    if !asm_file_name.ends_with(".txt") {
        return Err(FileHandlerError::ErrorInvalidExtension)
            .context("Output file must have a .txt extension.")
            .context(user_messages::USAGE_ERROR);
    }

    let bin_file = match File::options().read(true).open(bin_file_name) {
        Ok(file) => file,
        Err(_) => {
            return Err(FileHandlerError::ErrorFileOpenFailed)
                .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.")
                .context(user_messages::USAGE_ERROR);
        }
    };

    let mut asm_file = match File::options().write(true).create(true).open(asm_file_name) {
        Ok(file) => file,
        Err(_) => {
            return Err(FileHandlerError::ErrorFileOpenFailed)
                .context("Couldn't open or create the output file. Make sure the file is not write-protected if it already exists.")
                .context(user_messages::USAGE_ERROR);
        }
    };

    // Scan all labels into the symbol table
    // let symbol_table = read_labels(&bin_file)?;

    // Disassemble all the instructions and catch any errors
    // Write the disassembled instructions to the output file
    // write_output(&mut asm_file, &disassemble_instructions(&bin_file, &symbol_table))?;

    Ok(())
}

// Writes the disassembled instructions to the output ASM text file
fn write_output(asm_file: &mut File, disassembled_instructions: &Vec<String>) -> Result<()> {
    for instruction in disassembled_instructions {
        match asm_file.write_all(instruction.as_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(FileHandlerError::ErrorFileWriteFailed)
                .context("[INTERNAL ERROR] Couldn't write instructions to the assembly file.")
        }
    }

    Ok(())
}

// Scans the input machine code file for labels, and adds them to the symbol table for use later
fn read_labels(bin_file: &File) -> Result<SymbolTable> {
    // Stores all labels found in the file
    let mut symbol_table = symbol_table::new();

    let mut scanner = BufReader::new(bin_file);

    match scanner.rewind() {
        Ok(_) => (),
        Err(_) => return Err(FileHandlerError::ErrorFileRewindFailed)
            .context("[INTERNAL ERROR] Couldn't rewind the machine code file for symbol table pass.")
    }

    // Store the current label number
    let mut current_label: u16 = 0;
    
    // Read each instruction from the file
    loop {
        // Stores the current instruction
        let mut buffer = [0; 4];

        // Read 4-byte chunks of the file (instructions)
        match scanner.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(FileHandlerError::ErrorFileReadFailed)
                    .context("[INTERNAL ERROR] Couldn't read the machine code file for symbol table pass.")
            },
        }

        // Take the bytes and put them in a single u32, converting from network byte order if needed
        let instruction = u32::from_be_bytes(buffer);

        // If the instruction is a jump and its label is unique, add it to the symbol table
        if opcode_resolver::is_jump(extract_opcode(instruction)) {
            let label_address = extract_label(instruction);

            if !symbol_table.contains(label_address) {
                symbol_table.add_label(generate_label_name(current_label).as_str(), label_address);
                
                current_label += 1;
            }
        }
    }

    Ok(symbol_table)
}

// Gets a generic label name based on the given label number
fn generate_label_name(label_number: u16) -> String {
    format!("label_{}", label_number)
}

// Gets the first 8 bits of the instruction (the opcode)
fn extract_opcode(instruction: u32) -> u8 {
    ((instruction & 0xFF000000) >> 24) as u8
}

// Gets an indexed register operand from the instruction
fn extract_register(instruction: u32, index: usize) -> Result<u8> {
    if index > 2 {
        return Err(RegisterParseError::ErrorInvalidIndex)
            .context("[INTERNAL ERROR] Invalid register index access.")
    }

    // Grab the register from the instruction by masking out a 4-bit section (shifted by the index)
    Ok((instruction & (0x00F00000 >> (index * 4))) as u8)
}

// Gets the immediate operand from the instruction
fn extract_immediate(instruction: u32) -> u16 {
    (instruction & 0x0000FFFF) as u16
}

// Gets the label address from the instruction
// Functionally the same as extract_immediate() but included for clarity
fn extract_label(instruction: u32) -> u16 {
    extract_immediate(instruction)
}
