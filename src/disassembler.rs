use crate::errors::*;
use crate::instruction::disassembler_helpers::*;
use crate::instruction::instruction::*;
use crate::instruction::opcode_resolver;
use crate::utilities::symbol_table::SymbolTable;
use crate::utilities::*;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::{BufReader, ErrorKind, Read, Seek, Write};

// Initiates the disassembly of the given binary machine code file into an ASM text file
pub fn start_disassembler(bin_file_name: &str, asm_file_name: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !bin_file_name.ends_with(".bin") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Input file must have a .bin extension.")
            .context(user_messages::USAGE_ERROR);
    }

    // Open/create the input and output file
    if !asm_file_name.ends_with(".txt") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Output file must have a .txt extension.")
            .context(user_messages::USAGE_ERROR);
    }

    let bin_file = match File::options().read(true).open(bin_file_name) {
        Ok(file) => file,
        Err(_) => {
            return Err(FileHandlerError::FileOpenFailed)
                .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.")
                .context(user_messages::USAGE_ERROR);
        }
    };

    let mut asm_file = match File::options().write(true).create(true).open(asm_file_name) {
        Ok(file) => file,
        Err(_) => {
            return Err(FileHandlerError::FileOpenFailed)
                .context("Couldn't open or create the output file. Make sure the file is not write-protected if it already exists.")
                .context(user_messages::USAGE_ERROR);
        }
    };

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&bin_file)?;

    // Disassemble all the instructions and catch any errors
    // Write the disassembled instructions to the output file
    write_output(
        &mut asm_file,
        &disassemble_instructions(&bin_file, &symbol_table)?,
    )?;

    Ok(())
}

// Writes the disassembled instructions to the output ASM text file
fn write_output(asm_file: &mut File, disassembled_instructions: &Vec<String>) -> Result<()> {
    for instruction in disassembled_instructions {
        match asm_file.write_all(instruction.as_bytes()) {
            Ok(_) => (),
            Err(_) => {
                return Err(FileHandlerError::FileWriteFailed)
                    .context("[INTERNAL ERROR] Couldn't write instructions to the assembly file.")
            }
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
        Err(_) => {
            return Err(FileHandlerError::FileRewindFailed).context(
                "[INTERNAL ERROR] Couldn't rewind the machine code file for symbol table pass.",
            )
        }
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
                _ => return Err(FileHandlerError::FileReadFailed).context(
                    "[INTERNAL ERROR] Couldn't read the machine code file for symbol table pass.",
                ),
            },
        }

        // Take the bytes and put them in a single u32, converting from network byte order if needed
        let instruction = u32::from_be_bytes(buffer);

        // If the instruction is a jump and its label is unique, add it to the symbol table
        if opcode_resolver::is_jump(extract_opcode(instruction)) {
            let label_address = extract_address(instruction);

            if !symbol_table.contains(label_address) {
                symbol_table.add_label(generate_label_name(current_label).as_str(), label_address);

                current_label += 1;
            }
        }
    }

    Ok(symbol_table)
}

// Reads the machine code file and returns a Vec of the disassembled instructions
fn disassemble_instructions(bin_file: &File, symbol_table: &SymbolTable) -> Result<Vec<String>> {
    let mut scanner = BufReader::new(bin_file);
    scanner
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the machine code file for disassembler pass.")?;

    let mut disassembled_instructions = Vec::<String>::new();

    // Read each instruction from the file
    loop {
        // Stores the current instruction
        let mut buffer = [0; 4];

        // Read 4-byte chunks of the file (instructions)
        match scanner.read_exact(&mut buffer) {
            Ok(_) => (),
            Err(err) => match err.kind() {
                ErrorKind::UnexpectedEof => break,
                _ => return Err(FileHandlerError::FileReadFailed).context(
                    "[INTERNAL ERROR] Couldn't read the machine code file for symbol table pass.",
                ),
            },
        }

        // Take the bytes and put them in a single u32, converting from network byte order if needed
        let encoded_instruction = u32::from_be_bytes(buffer);

        // Gets an InstructionContainer with the necessary format and the given opcode
        let mut instruction =
            match opcode_resolver::get_instruction_container(extract_opcode(encoded_instruction)) {
                Some(container) => container,
                None => {
                    return Err(OpcodeParseError::UnknownOpcode)
                        .context("Invalid instruction found in the machine code file.")
                }
            };

        // Populate the fields of the container with the data from the instruction
        instruction.decode(encoded_instruction);

        // Disassemble the instruction into a String and add it to the Vec
        disassembled_instructions.push(instruction.disassemble(symbol_table)?);
    }

    Ok(disassembled_instructions)
}
