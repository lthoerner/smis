use super::super::errors::*;
use super::super::utilities::string_methods::*;
use super::super::utilities::symbol_table::*;
use super::assembler_helpers::*;
use super::disassembler_helpers::*;
use super::opcode_resolver::*;
use anyhow::{Context, Result};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Instruction {
    // Assembles an instruction from a string into a format struct
    fn assemble(&mut self, instruction_text: &str, symbol_table: &SymbolTable) -> Result<()>;
    // Disassembles an instruction from a format struct into a string
    fn disassemble(&self, symbol_table: &SymbolTable) -> Result<String>;
    // Encodes an instruction from a format struct into a u32
    fn encode(&self) -> u32;
    // Decodes an instruction from a u32 into a format struct
    fn decode(&mut self, encoded_instruction: u32);
}

// Allows for generic referencing of instruction format structs
#[enum_dispatch(Instruction)]
#[derive(Debug)]
pub enum InstructionContainer {
    RFormat,
    IFormat,
    JFormat,
}

// Instruction format structs
#[derive(Default, Debug)]
pub struct RFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub r_op2: u8,
}

#[derive(Default, Debug)]
pub struct IFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub i_op2: u16,
}

#[derive(Default, Debug)]
pub struct JFormat {
    pub opcode: u8,
    pub dest_addr: u16,
}

// See trait for function explanations
impl Instruction for RFormat {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE instructions do not have an destination register
        // This could be renamed to compare_mode, but there could eventually be
        // other instructions that also have no destination register
        let has_dest = has_dest(self.opcode);
        // Similarly, NOT and COPY instructions do not have a second operand register
        let has_op2 = has_reg_op2(self.opcode);

        // In the case of a missing destination register, all the operand words are shifted over to the left
        let missing_destination_index_adjustment = !has_dest as usize;

        // If there is no destination register, the r_dest field is left blank
        if has_dest {
            self.r_dest = get_register(instruction_text, 1)?;
        } else {
            self.r_dest = 0x00;
        };

        // All R-Format instructions are guaranteed to have a first operand register
        self.r_op1 = get_register(instruction_text, 2 - missing_destination_index_adjustment)?;

        // If there is no second operand register, the r_op2 field is left blank
        if has_op2 {
            self.r_op2 = get_register(instruction_text, 3 - missing_destination_index_adjustment)?;
        } else {
            self.r_op2 = 0x00;
        }

        Ok(())
    }

    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_string = String::new();

        // Concatenate the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnem) => mnem,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Concatenate the destination register
        if has_dest(self.opcode) {
            instruction_string.push_str(&format_register(self.r_dest)?);
            instruction_string.push(' ');
        }

        // Concatenate the first operand register
        instruction_string.push_str(&format_register(self.r_op1)?);
        instruction_string.push(' ');

        // Concatenate the second operand register
        if has_reg_op2(self.opcode) {
            instruction_string.push_str(&format_register(self.r_op2)?);
        }

        instruction_string.push('\n');

        Ok(instruction_string)
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.r_dest as u32) << 20
            | (self.r_op1 as u32) << 16
            | (self.r_op2 as u32) << 12
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.r_dest = extract_register(encoded_instruction, 0);
        self.r_op1 = extract_register(encoded_instruction, 1);
        self.r_op2 = extract_register(encoded_instruction, 2);
    }
}

// See trait for function explanations
impl Instruction for IFormat {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE-IMM instructions do not have an destination register
        let has_dest = has_dest(self.opcode);
        // Similarly, SET instructions do not have a register operand
        let has_reg_op = has_reg_op1(self.opcode);

        // If there is no destination register, the r_dest field is left blank
        self.r_dest = match has_dest {
            true => get_register(instruction_text, 1)?,
            false => 0x00,
        };

        // If there is no register operand, the r_op1 field is left blank
        self.r_op1 = match has_reg_op {
            true => get_register(instruction_text, 2)?,
            false => 0x00,
        };

        // All I-Format instructions are guaranteed to have an immediate operand
        self.i_op2 = get_immediate(instruction_text)?;

        Ok(())
    }

    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_string = String::new();

        // Concatenate the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnem) => mnem,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Concatenate the destination register
        if has_dest(self.opcode) {
            instruction_string.push_str(&format_register(self.r_dest)?);
            instruction_string.push(' ');
        }

        // Concatenate the register operand
        if has_reg_op1(self.opcode) {
            instruction_string.push_str(&format_register(self.r_op1)?);
            instruction_string.push(' ');
        }

        // Concatenate the immediate operand
        instruction_string.push_str(&format_immediate(self.i_op2));

        instruction_string.push('\n');

        Ok(instruction_string)
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.r_dest as u32) << 20
            | (self.r_op1 as u32) << 16
            | (self.i_op2 as u32)
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.r_dest = extract_register(encoded_instruction, 0);
        self.r_op1 = extract_register(encoded_instruction, 1);
        self.i_op2 = extract_immediate(encoded_instruction);
    }
}

// See trait for function explanations
impl Instruction for JFormat {
    fn assemble(&mut self, instruction_text: &str, symbol_table: &SymbolTable) -> Result<()> {
        // HALT instructions do not have a destination label
        let has_label = has_label(self.opcode);

        // Skip address resolution for instructions with no destination label
        if has_label {
            let label = instruction_text.without_first_word();

            // Get the label address of a given label name, if it is not a HALT
            self.dest_addr = match symbol_table.find_address(label.trim()) {
                Some(addr) => addr,
                None => {
                    return Err(SymbolTableError::LabelNotFound)
                        .context("Label not found in symbol table.")
                        .context(format!("At: '{}'", label))
                }
            };
        }

        Ok(())
    }

    fn disassemble(&self, symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_string = String::new();

        // Concatenate the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnem) => mnem,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Concatenate the jump label
        if has_label(self.opcode) {
            let label = match symbol_table.find_name(self.dest_addr) {
                Some(label) => label,
                None => {
                    return Err(SymbolTableError::LabelNotFound)
                        .context("[INTERNAL ERROR] Label not found in symbol table.")
                }
            };

            instruction_string.push_str(label);
        }

        instruction_string.push('\n');

        Ok(instruction_string)
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24 | (self.dest_addr as u32)
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.dest_addr = extract_address(encoded_instruction);
    }
}
