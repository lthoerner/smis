use crate::errors::*;
use crate::utilities::*;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::Write;

pub fn start_disassembler(bin_file_name: &str, asm_file_name: &str) -> Result<()> {
    if !bin_file_name.ends_with(".bin") {
        return Err(FileHandlerError::ErrorInvalidExtension)
            .context("Input file must have a .bin extension.")
            .context(user_messages::USAGE_ERROR);
    }

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

    // write_output(&mut asm_file, &disassemble_instructions(&bin_file, &symbol_table))?;

    Ok(())
}

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
