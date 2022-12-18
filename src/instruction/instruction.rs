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
    // Disassembles an instruction from a u32 into a format struct
    fn disassemble(&self) -> Result<String>;
    // Encodes an instruction from a format struct into a u32
    fn encode(&self) -> u32;
    // Decodes an instruction from a format struct into a string
    fn decode(&mut self, encoded_instruction: u32);
}

// Allows for generic referencing of instruction format structs
#[enum_dispatch(Instruction)]
pub enum InstructionContainer {
    RFormat,
    IFormat,
    JFormat,
}

#[derive(Default)]
pub struct RFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub r_op2: u8,
}

#[derive(Default)]
pub struct IFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub i_op2: u16,
}

#[derive(Default)]
pub struct JFormat {
    pub opcode: u8,
    pub dest_addr: u16,
}

// See trait for function explanations
impl Instruction for RFormat {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE instructions do not have an destination register
        // This could be renamed to compare_mode, but there could eventually be other instructions that also have
        // no destination register, so this is a more modular approach
        let mut no_dest = false;
        // Similarly, NOT and COPY instructions do not have a second operand register
        let mut no_op2 = false;

        match self.opcode {
            OP_COMPARE => no_dest = true,
            OP_NOT | OP_COPY => no_op2 = true,
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

        Ok(())
    }

    fn disassemble(&self) -> Result<String> {
        Ok(String::from("blah"))
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
        self.r_op2 = extract_register(encoded_instruction, 1);
    }
}

// See trait for function explanations
impl Instruction for IFormat {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE-IMM instructions do not have an destination register
        let mut no_dest = false;
        // Similarly, SET instructions do not have a register operand
        let mut no_reg_op = false;

        match self.opcode {
            OP_COMPARE_IMM => no_dest = true,
            OP_SET => no_reg_op = true,
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

        Ok(())
    }

    fn disassemble(&self) -> Result<String> {
        Ok(String::from("blah"))
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
        let mut no_label = false;

        match self.opcode {
            OP_HALT => no_label = true,
            // Use default value for instructions with standard format
            _ => (),
        }

        // Skip address resolution for instructions with no destination label
        if !no_label {
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

        Ok(())
    }

    fn disassemble(&self) -> Result<String> {
        Ok(String::from("blah"))
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24 | (self.dest_addr as u32)
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.dest_addr = extract_address(encoded_instruction);
    }
}
