use crate::errors::*;
use crate::utilities::instruction::*;
use crate::utilities::string_methods::SMISString;
use crate::utilities::symbol_table::SymbolTable;
use crate::utilities::*;
use anyhow::Context;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, Write};

// Initiates the assembly of the given ASM text file into a binary machine code file
pub fn start_assembler(asm_file_name: &str, bin_file_name: &str) -> Result<()> {
    // Ensure the input and output files have the correct extensions
    if !asm_file_name.ends_with(".txt") {
        return Err(FileHandlerError::ErrorInvalidExtension)
            .context("Input file must have a .txt extension.")
            .context(user_messages::USAGE_ERROR);
    }

    if !bin_file_name.ends_with(".bin") {
        return Err(FileHandlerError::ErrorInvalidExtension)
            .context("Output file must have a .bin extension.")
            .context(user_messages::USAGE_ERROR);
    }

    // Open/create the input and output file
    let asm_file = File::options().read(true).open(asm_file_name)
        .map_err(|_| FileHandlerError::ErrorFileOpenFailed)
        .context("Couldn't open the input file. Make sure the file exists and is in the necessary directory.")
        .context(user_messages::USAGE_ERROR)?;

    let mut bin_file = File::options().write(true).create(true).open(bin_file_name)
        .map_err(|_| FileHandlerError::ErrorFileCreateFailed)
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
        bin_file.write_all(&instruction.to_be_bytes())
            .map_err(|_| FileHandlerError::ErrorFileWriteFailed)
            .context("[INTERNAL ERROR] Couldn't write instructions to the binary file.")?;
    }

    Ok(())
}

// Scans the input ASM file for labels, and adds them to the symbol table for use later
fn read_labels(asm_file: &File) -> Result<SymbolTable> {
    // Stores all labels found in the file along with their corresponding instruction addressses
    let mut symbol_table = symbol_table::new();

    let mut scanner = BufReader::new(asm_file);

    scanner.rewind()
        .map_err(|_| FileHandlerError::ErrorFileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for symbol table pass.")?;

    // Store the address of the instruction currently being scanned
    let mut current_address: u16 = 0x00;

    // For each line in the file
    for line in scanner.lines() {
        // Handle any errors for line reading
        let line = line
            .map_err(|_| FileHandlerError::ErrorFileReadFailed)
            .context("[INTERNAL ERROR] Couldn't read a line from the ASM file for symbol table pass.")?
            .as_str();

        // Add any labels to the symbol table
        if is_label(line) {
            // TODO: Add get_label_name() and refactor
            symbol_table.add_label(
                match line.trim().strip_suffix(':') {
                    Some(name) => name,
                    // This should never happen, as the above condition requires the line to end in ':'
                    None => {
                        return Err(SymbolTableError::ErrorCouldNotAddLabel)
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
    scanner.rewind()
        .map_err(|_| FileHandlerError::ErrorFileRewindFailed)
        .context("[INTERNAL ERROR] Couldn't rewind the ASM file for assembler pass.")?;

    let mut assembled_instructions = Vec::<u32>::new();

    // Line count is stored to give more descriptive error messages
    let mut line_count: u16 = 0;

    // For each line in the file
    for line in scanner.lines() {
        line_count += 1;

        // Handle any errors for line reading
        let line = line
            .map_err(|_| FileHandlerError::ErrorFileReadFailed)
            .context("[INTERNAL ERROR] Couldn't read a line from the ASM file for the assembler pass.")?
            .trim();

        // Skip non-instruction lines
        if is_blankline(line) || is_comment(line) || is_label(line) {
            continue;
        }

        let opcode = parse_opcode(line)
            .context(format!("On line: {}", line_count))?;

        // Gets an Instruction with the necessary format and the given opcode
        let instruction =
            match opcode_resolver::get_instruction_container(opcode) {
                Some(instr) => instr,
                None => return Err(OpcodeParseError::ErrorUnknownOpcode).context(
                    "[INTERNAL ERROR] Used an invalid opcode to create an instruction container.",
                ),
            };

        // Assemble the instruction and add it to the Vec
        assembled_instructions.push(match instruction {
            InstructionContainer::RFormat(container) =>
                container.assemble(line)
                    .context(format!("On line: {}", line_count))?.encode(),
            InstructionContainer::IFormat(container) =>
                container.assemble(line)
                    .context(format!("On line: {}", line_count))?.encode(),
            InstructionContainer::JFormat(container) =>
                container.assemble(line, symbol_table)
                    .context(format!("On line: {}", line_count))?.encode(),
        });
    }

    Ok(assembled_instructions)
}

// TODO: Add anyhow error handling
impl instruction::RFormat {
    // Assembles all R-Format instructions into a u32
    fn assemble(mut self, instruction_text: &str) -> Result<Self> {
        // COMPARE instructions do not have an destination register
        // This could be renamed to compare_mode, but there could eventually be other instructions that also have
        // no destination register, so this is a more modular approach
        let mut no_dest = false;
        // Similarly, NOT and COPY instructions do not have a second operand register
        let mut no_op2 = false;

        match self.opcode {
            opcode_resolver::OP_COMPARE => no_dest = true,
            opcode_resolver::OP_NOT | opcode_resolver::OP_COPY => no_op2 = true,
            // Use default values for instructions with standard format
            _ => (),
        }

        // In the case of a missing destination register, all the operand words are shifted over to the left
        let missing_destination_index_adjustment = no_dest as usize;

        // If there is no destination register, the r_dest field is left blank
        self.r_dest = match no_dest {
            true => 0x00,
            false => get_register(instruction_text, 1)?,
        };

        // All R-Format instructions are guaranteed to have a first operand register
        self.r_op1 = get_register(instruction_text, 2 - missing_destination_index_adjustment)?;

        // If there is no second operand register, the r_op2 field is left blank
        self.r_op2 = match no_op2 {
            true => 0x00,
            false => get_register(instruction_text, 3 - missing_destination_index_adjustment)?,
        };

        Ok(self)
    }
}

impl instruction::IFormat {
    // Assembles all I-Format instructions into a u32
    fn assemble(mut self, instruction_text: &str) -> Result<Self> {
        // COMPARE-IMM instructions do not have an destination register
        let mut no_dest = false;
        // Similarly, SET instructions do not have a register operand
        let mut no_reg_op = false;

        match self.opcode {
            opcode_resolver::OP_COMPARE_IMM => no_dest = true,
            opcode_resolver::OP_SET => no_reg_op = true,
            // Use default values for instructions with standard format
            _ => (),
        }

        // If there is no destination register, the r_dest field is left blank
        self.r_dest = match no_dest {
            true => 0x00,
            false => get_register(instruction_text, 1)?,
        };

        // If there is no register operand, the r_op1 field is left blank
        self.r_op1 = match no_reg_op {
            true => 0x00,
            false => get_register(instruction_text, 2)?,
        };

        // All I-Format instructions are guaranteed to have an immediate operand
        self.i_op2 = get_immediate(instruction_text)?;

        Ok(self)
    }
}

impl instruction::JFormat {
    // Assembles all J-Format instructions into a u32
    fn assemble(mut self, instruction_text: &str, symbol_table: &SymbolTable) -> Result<Self> {
        // HALT instructions do not have a destination label
        let mut no_label = false;

        match self.opcode {
            opcode_resolver::OP_HALT => no_label = true,
            // Use default value for instructions with standard format
            _ => (),
        }

        // Skip address resolution for instructions with no destination label
        match no_label {
            true => (),
            false => {
                let label = instruction_text.without_first_word();

                // Get the label address of a given label name, if it is not a HALT
                self.dest_addr = match symbol_table.find_address(label.trim()) {
                    Some(addr) => addr,
                    None => {
                        return Err(SymbolTableError::ErrorLabelNotFound)
                            .context("Label not found in symbol table.")
                            .context(format!("At: '{}'", label))
                    }
                };
            }
        }

        Ok(self)
    }
}

// Takes the instruction, gets the mnemonic, and translates it into an opcode
fn parse_opcode(instruction: &str) -> Result<u8> {
    match instruction.get_word(0) {
        Some(mnemonic) => match opcode_resolver::get_opcode(mnemonic) {
            Some(opcode) => Ok(opcode),
            None => Err(MnemonicParseError::ErrorUnknownMnemonic)
                .context("Unknown or malformed mnemonic.")
                .context(format!("At: '{}'", mnemonic)),
        },
        None => Err(MnemonicParseError::ErrorInvalidIndex)
            .context("[INTERNAL ERROR] Invalid mnemonic index access."),
    }
}

// Gets the register number operand from a given instruction by pulling the
// given operand with get_word() and parsing it using parse_register()
fn get_register(instruction: &str, index: usize) -> Result<u8> {
    match instruction.get_word(index) {
        Some(unparsed_register) => parse_register(unparsed_register)
            .context(format!("At: '{}'", unparsed_register)),
        None => Err(RegisterParseError::ErrorInvalidIndex)
            .context("[INTERNAL ERROR] Invalid register index access."),
    }
}

// Parses a register number from a string to a u8
fn parse_register(register: &str) -> Result<u8> {
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
        None => {
            return Err(RegisterParseError::ErrorInvalidPrefix).context("Invalid register prefix.")
        }
    };

    // TODO: Different error message for out of u8 bounds
    // Make sure the value after the prefix is numerical and within u8 bounds
    let register_num = trimmed_register.parse::<u8>()
        .map_err(|_| RegisterParseError::ErrorNonNumeric)
        .context("Non-numeric register number.")?;

    // Make sure the register exists (0-15)
    match register_num > 15 {
        true => Err(RegisterParseError::ErrorInvalidNumber)
            .context("Register number out of bounds (0-15).")
            .context(format!("At: '{}'", register)),
        false => Ok(register_num),
    }
}

// Gets the immediate value operand from a given instruction by pulling the
// last operand with get_word() and parsing it using parse_immediate()
fn get_immediate(instruction: &str) -> Result<u16> {
    // TODO: There could be more words between other operands and the immediate operand
    // Gets the last word of the line and attempts to parse it into an immediate value
    match instruction.get_word(instruction.count_words() - 1) {
        Some(unparsed_immediate) => parse_immediate(unparsed_immediate)
            .context(format!("At: '{}'", unparsed_immediate)),
        None => Err(ImmediateParseError::ErrorInvalidIndex)
            .context("[INTERNAL ERROR] Invalid immediate index access."),
    }
}

// Parses an immediate value from a string to a u16
fn parse_immediate(immediate: &str) -> Result<u16> {
    // Make sure the immediate begins with '#'
    let trimmed_immediate = match immediate.strip_prefix('#') {
        Some(trim) => trim,
        None => {
            return Err(ImmediateParseError::ErrorInvalidPrefix)
                .context("Invalid immediate prefix.")
        }
    };

    // Make sure the value after the prefix is numerical and within u16 bounds, then return it
    trimmed_immediate.parse::<u16>()
        .map_err(|_| ImmediateParseError::ErrorNonNumeric)
        .context("Non-numeric immediate value.")
}

// Checks whether a given string ends with a ':', denoting that it is a jump label
fn is_label(line: &str) -> bool {
    // Forbids labels from containing whitespace
    if line.count_words() > 1 {
        return false;
    }

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
