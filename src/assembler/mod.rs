mod errors;

use std::fs::File;
use std::io::{ BufRead, BufReader, Seek, Write };
use crate::utilities::*;
use crate::utilities::instruction::Instruction;
use crate::utilities::symbol_table::SymbolTable;
use crate::utilities::string_methods::SMISString;
use self::errors::assembler_error::*;


pub fn start_assembler(asm_file_name: &str, bin_file_name: &str) -> Result<(), FileHandlerError> {
    if !asm_file_name.ends_with(".txt") {
        panic!("Input file must have a .txt extension.");
    }

    if !bin_file_name.ends_with("bin") {
        panic!("Output file must have a .bin extension.");
    }

    // Open the input and output file
    let asm_file = File::options().read(true).open(asm_file_name)?;
    let mut bin_file = File::options().write(true).create(true).open(bin_file_name)?;

    // Scan all labels into the symbol table
    let symbol_table = read_labels(&asm_file)?;

    // Assemble all the instructions and catch any errors
    match assemble_instructions(&asm_file, &symbol_table) {
        Ok(assembled_instructions) => write_output(&mut bin_file, &assembled_instructions)?,
        Err(_) => panic!("Something went wrong.")
    };

    Ok(())
}

fn write_output(bin_file: &mut File, assembled_instructions: &Vec<u32>) -> Result<(), FileHandlerError> {
    for instruction in assembled_instructions {
        let instruction = *instruction;

        match bin_file.write_all(&instruction.to_be_bytes()) {
            Ok(_) => (),
            Err(_) => return Err(FileHandlerError::ErrorFileWriteFailed)
        };
    }

    Ok(())
}

// Scans the input ASM file for labels, and adds them to the symbol table for use later
fn read_labels(asm_file: &File) -> Result<SymbolTable, FileHandlerError> {
    // Store the address of the instruction currently being scanned
    let mut current_address: u16 = 0x00;
    
    // Stores all labels found in the file
    let mut symbol_table = symbol_table::new();
    
    let mut scanner = BufReader::new(asm_file);
    
    match scanner.rewind() {
        Ok(()) => (),
        Err(_) => return Err(FileHandlerError::ErrorInvalidFileContents)
    };

    // For each line in the file
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

// Reads the ASM file and returns a Vec of the assembled instructions
fn assemble_instructions(asm_file: &File, symbol_table: &SymbolTable) -> Result<Vec<u32>, AssemblerError> {
    let mut scanner = BufReader::new(asm_file);
    match scanner.rewind() {
        Ok(()) => (),
        Err(_) => return Err(AssemblerError::from(FileHandlerError::ErrorInvalidFileContents))
    };

    let mut assembled_instructions = Vec::<u32>::new();

    // Line count is stored to give more descriptive error messages
    let mut line_count: u16 = 0;

    // For each line in the file
    for line in scanner.lines() {
        line_count += 1;

        let line = match line {
            Ok(text) => text,
            Err(_) => return Err(AssemblerError::from(FileHandlerError::ErrorInvalidFileContents))
        };

        // Trim any whitespace from the instruction for parsing
        let line = line.trim();

        // Skip non-instruction lines
        if is_blankline(line) || is_comment(line) || is_label(line) { continue; }

        let opcode = parse_opcode(line)?;

        // Gets an Instruction with the necessary format and the given opcode
        let instruction = match opcode_resolver::get_instruction_format(opcode) {
            Some(instr) => instr,
            None => return Err(AssemblerError::from(MnemonicParseError::ErrorUnknownMnemonic))
        };

        // Assemble the instruction and add it to the Vec
        assembled_instructions.push(match instruction {
            Instruction::RFormat {..} => assemble_r_format(line, instruction)?,
            Instruction::IFormat {..} => assemble_i_format(line, instruction)?,
            Instruction::JFormat {..} => assemble_j_format(line, instruction, symbol_table)?
        });

        // print!("[{:02}] ", line_count);
        // match instruction {
        //     Instruction::RFormat {..} => println!("{:<40} is an R-Format: 0x{:08X}", line, assemble_r_format(line, instruction)?),
        //     Instruction::IFormat {..} => println!("{:<40} is an I-Format: 0x{:08X}", line, assemble_i_format(line, instruction)?),
        //     Instruction::JFormat {..} => println!("{:<40} is a J-Format:  0x{:08X}", line, assemble_j_format(line, instruction, &symbol_table)?)
        // }
    }

    Ok(assembled_instructions)
}

// Assembles all R-Format instructions into a u32
fn assemble_r_format(instruction: &str, mut instruction_container: Instruction) -> Result<u32, ParseError> {
    // COMPARE instructions do not have an destination register
    // This could be renamed to compare_mode, but there could eventually be other instructions that also have
    // no destination register, so this is a more modular approach
    let mut no_dest = false;
    // Similarly, NOT and COPY instructions do not have a second operand register
    let mut no_op2 = false;

    // Make sure that the Instruction passed in is an R-Format
    match instruction_container {
        Instruction::RFormat { opcode, ref mut r_dest, ref mut r_op1, ref mut r_op2 } => {
            match opcode {
                opcode_resolver::OP_COMPARE => no_dest = true,
                opcode_resolver::OP_NOT | opcode_resolver::OP_COPY => no_op2 = true,
                // Use default values for instructions with standard format
                _ => ()
            }

            // In the case of a missing destination register, all the operand words are shifted over to the left
            let missing_destination_index_adjustment = no_dest as usize;

            // If there is no destination register, the r_dest field is left blank
            *r_dest = match no_dest {
                true => 0x00,
                false => get_register(instruction, 1)?
            };

            // All R-Format instructions are guaranteed to have a first operand register
            *r_op1 = get_register(instruction, 2 - missing_destination_index_adjustment)?;

            // If there is no second operand register, the r_op2 field is left blank
            *r_op2 = match no_op2 {
                true => 0x00,
                false => get_register(instruction, 3 - missing_destination_index_adjustment)?
            };

            Ok(instruction_container.encode())
        },
        // These should never happen, as the function is only called from a match block that switches based on the enum variant
        Instruction::IFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble I-Format instruction as R-Format"),
        Instruction::JFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble J-Format instruction as R-Format")
    }
}

// Assembles all I-Format instructions into a u32
fn assemble_i_format(instruction: &str, mut instruction_container: Instruction) -> Result<u32, ParseError> {
    // COMPARE-IMM instructions do not have an destination register
    let mut no_dest = false;
    // Similarly, SET instructions do not have a register operand
    let mut no_reg_op = false;

    // Make sure that the Instruction passed in is an I-Format
    match instruction_container {
        Instruction::IFormat { opcode, ref mut r_dest, ref mut r_op1, ref mut i_op2 } => {
            match opcode {
                opcode_resolver::OP_COMPARE_IMM => no_dest = true,
                opcode_resolver::OP_SET => no_reg_op = true,
                // Use default values for instructions with standard format
                _ => ()
            }

            // If there is no destination register, the r_dest field is left blank
            *r_dest = match no_dest {
                true => 0x00,
                false => get_register(instruction, 1)?
            };

            // If there is no register operand, the r_op1 field is left blank
            *r_op1 = match no_reg_op {
                true => 0x00,
                false => get_register(instruction, 2)?
            };

            // All I-Format instructions are guaranteed to have an immediate operand
            *i_op2 = get_immediate(instruction)?;

            Ok(instruction_container.encode())
        },
        // These should never happen, as the function is only called from a match block that switches based on the enum variant
        Instruction::RFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble R-Format instruction as I-Format"),
        Instruction::JFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble J-Format instruction as I-Format")
    }
}

// Assembles all J-Format instructions into a u32
fn assemble_j_format(instruction: &str, mut instruction_container: Instruction, symbol_table: &SymbolTable) -> Result<u32, ParseError> {
    // HALT instructions do not have a destination label
    let mut no_label = false;

    // Make sure that the Instruction passed in is a J-Format
    match instruction_container {
        Instruction::JFormat { opcode, ref mut dest_addr } => {
            match opcode {
                opcode_resolver::OP_HALT => no_label = true,
                // Use default value for instructions with standard format
                _ => ()
            }

            // Skip address resolution for instructions with no destination label
            match no_label {
                true => (),
                false => {
                    let label = match instruction.without_first_word() {
                        Some(lbl) => lbl,
                        None => return Err(ParseError::from(LabelParseError::ErrorMalformedLabel))
                    };

                    // Get the label address of a given label name, if it is not a HALT
                    *dest_addr = match symbol_table.find_address(label.as_str()) {
                        Some(addr) => addr,
                        None => return Err(ParseError::from(LabelParseError::ErrorLabelNotFound))
                    };
                }
            }

            Ok(instruction_container.encode())
        },
        // These should never happen, as the function is only called from a match block that switches based on the enum variant
        Instruction::RFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble R-Format instruction as J-Format"),
        Instruction::IFormat {..} => panic!("[INTERNAL ERROR] Attempted to assemble I-Format instruction as J-Format")
    }
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
    let unparsed_immediate = match instruction.get_word(instruction.count_words() - 1) {
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
    // Forbids labels from containing whitespace
    if line.count_words() > 1 { return false; }

    // TODO: Possibly add checking for extra ':' at the end
    !is_comment(line) && line.chars().last() == Some(':')
}

// Checks whether a given string starts with a "//", denoting that it is a comment
fn is_comment(line: &str) -> bool {
    line.trim().starts_with("//")
}

// Checks whether a given string is only whitespace, denoting that it is a blank line
fn is_blankline(line: &str) -> bool {
    line.chars().all(|c| c.is_whitespace())
}