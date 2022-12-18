use crate::errors::*;
use crate::instruction::assembler_helpers::*;
use crate::instruction::instruction::Instruction;
use crate::instruction::*;
use crate::utilities::symbol_table::SymbolTable;
use crate::utilities::*;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, Write};

// Initiates the assembly of the given ASM text file into a binary machine code file
pub fn start_assembler(asm_file_name: &str, bin_file_name: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !asm_file_name.ends_with(".txt") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Input file must have a .txt extension.")
            .context(user_messages::USAGE_ERROR);
    }

    if !bin_file_name.ends_with(".bin") {
        return Err(FileHandlerError::InvalidExtension)
            .context("Output file must have a .bin extension.")
            .context(user_messages::USAGE_ERROR);
    }

    // Open/create the input and output file
    let asm_file = File::options().read(true).open(asm_file_name)
        .map_err(|_| FileHandlerError::FileOpenFailed)
        .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.")
        .context(user_messages::USAGE_ERROR)?;

    let mut bin_file = File::options().write(true).create(true).open(bin_file_name)
        .map_err(|_| FileHandlerError::FileCreateFailed)
        .context("Couldn't open or create the output file. Make sure the file is not write-protected if it already exists.")
        .context(user_messages::USAGE_ERROR)?;

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&asm_file)?;

    // Assemble all the instructions and catch any errors
    // Write the assembled instructions to the output file
    write_output(
        &mut bin_file,
        &assemble_instructions(&asm_file, &symbol_table)?,
    )?;

    Ok(())
}

// Writes the assembled instructions to the output machine code file
fn write_output(bin_file: &mut File, assembled_instructions: &Vec<u32>) -> Result<()> {
    for &instruction in assembled_instructions {
        // Instruction is converted to big-endian (network byte order) before being written to the file
        bin_file
            .write_all(&instruction.to_be_bytes())
            .map_err(|_| FileHandlerError::FileWriteFailed)
            .context("[INTERNAL ERROR] Couldn't write instructions to the binary file.")?;
    }

    Ok(())
}

// Scans the input ASM file for labels, and adds them to the symbol table for use later
fn read_labels(asm_file: &File) -> Result<SymbolTable> {
    // Stores all labels found in the file along with their corresponding instruction addressses
    let mut symbol_table = symbol_table::new();

    let mut scanner = BufReader::new(asm_file);
    scanner
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for symbol table pass.")?;

    // Store the address of the instruction currently being scanned
    let mut current_address: u16 = 0x00;

    // For each line in the file
    for line in scanner.lines() {
        // Handle any errors for line reading
        let line = line
            .map_err(|_| FileHandlerError::FileReadFailed)
            .context(
                "[INTERNAL ERROR] Couldn't read a line from the ASM file for symbol table pass.",
            )?;

        let line = line.trim();

        // Add any labels to the symbol table
        if is_label(line) {
            // TODO: Add get_label_name() and refactor
            symbol_table.add_label(
                match line.trim().strip_suffix(':') {
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
fn assemble_instructions(asm_file: &File, symbol_table: &SymbolTable) -> Result<Vec<u32>> {
    let mut scanner = BufReader::new(asm_file);
    scanner
        .rewind()
        .map_err(|_| FileHandlerError::FileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for assembler pass.")?;

    let mut assembled_instructions = Vec::<u32>::new();

    // Line count is stored to give more descriptive error messages
    let mut line_count: u16 = 0;

    // For each line in the file
    for line in scanner.lines() {
        line_count += 1;

        // Handle any errors for line reading
        let line = line
            .map_err(|_| FileHandlerError::FileReadFailed)
            .context(
                "[INTERNAL ERROR] Couldn't read a line from the ASM file for the assembler pass.",
            )?;

        let line = line.trim();

        // Skip non-instruction lines
        if is_blankline(line) || is_comment(line) || is_label(line) {
            continue;
        }

        let opcode =
            get_opcode_from_instruction_text(line).context(format!("On line: {}", line_count))?;

        // Gets an InstructionContainer with the necessary format and the given opcode
        let mut instruction =
            match opcode_resolver::get_instruction_container(opcode) {
                Some(instr) => instr,
                None => return Err(OpcodeParseError::UnknownOpcode).context(
                    "[INTERNAL ERROR] Used an invalid opcode to create an instruction container.",
                ),
            };

        // Assemble the instruction and add it to the Vec
        instruction
            .assemble(line, symbol_table)
            .context(format!("On line: {}", line_count))?;

        // Encode the assembled instruction and add it to the Vec
        assembled_instructions.push(instruction.encode());
    }

    Ok(assembled_instructions)
}
