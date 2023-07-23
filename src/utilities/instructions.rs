use super::errors::*;
use crate::assembler::*;
use crate::disassembler::*;
use crate::utilities::{
    opcode_utilities::*, string_methods::SmisString, symbol_table::SymbolTable,
};
use anyhow::{Context, Result};

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

// Instruction format structs
#[derive(Default, Debug)]
pub struct RTypeInstruction {
    pub opcode: u8,
    pub destination_register: u8,
    pub operand_1_register: u8,
    pub operand_2_register: u8,
}

impl RTypeInstruction {
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            ..Self::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct ITypeInstruction {
    pub opcode: u8,
    pub destination_register: u8,
    pub operand_1_register: u8,
    pub operand_2_immediate: u16,
}

impl ITypeInstruction {
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            ..Self::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct JTypeInstruction {
    pub opcode: u8,
    pub destination_memory_address: u16,
}

impl JTypeInstruction {
    pub fn new(opcode: u8) -> Self {
        Self {
            opcode,
            ..Self::default()
        }
    }
}

// See trait for method descriptions
impl Instruction for RTypeInstruction {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE instructions do not have an destination register
        // This could be renamed to compare_mode, but there could eventually be
        // other instructions that also have no destination register
        let has_destination_register = has_destination_register(self.opcode);
        // Similarly, NOT and COPY instructions do not have a second operand register
        let has_operand_2_register = has_operand_2_register(self.opcode);

        // In the case of a missing destination register,
        // all the operand words are shifted over to the left
        let missing_destination_index_adjustment = !has_destination_register as usize;

        // If there is no destination register, the destination_register field is left blank
        if has_destination_register {
            self.destination_register = get_register(instruction_text, 1)?;
        }

        // All R-Type instructions are guaranteed to have a first operand register
        self.operand_1_register =
            get_register(instruction_text, 2 - missing_destination_index_adjustment)?;

        // If there is no second operand register, the operand_2_register field is left blank
        if has_operand_2_register {
            self.operand_2_register =
                get_register(instruction_text, 3 - missing_destination_index_adjustment)?;
        }

        Ok(())
    }

    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_string = String::new();

        // Append the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnemonic) => mnemonic,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Append the destination register
        if has_destination_register(self.opcode) {
            instruction_string.push_str(&format_register(self.destination_register)?);
            instruction_string.push(' ');
        }

        // Append the first operand register
        instruction_string.push_str(&format_register(self.operand_1_register)?);
        instruction_string.push(' ');

        // Append the second operand register
        if has_operand_2_register(self.opcode) {
            instruction_string.push_str(&format_register(self.operand_2_register)?);
        }

        instruction_string.push('\n');

        Ok(instruction_string)
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.destination_register as u32) << 20
            | (self.operand_1_register as u32) << 16
            | (self.operand_2_register as u32) << 12
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.destination_register = extract_register(encoded_instruction, 0);
        self.operand_1_register = extract_register(encoded_instruction, 1);
        self.operand_2_register = extract_register(encoded_instruction, 2);
    }
}

// See trait for method descriptions
impl Instruction for ITypeInstruction {
    fn assemble(&mut self, instruction_text: &str, _symbol_table: &SymbolTable) -> Result<()> {
        // COMPARE-IMM instructions do not have an destination register
        let has_destination_register = has_destination_register(self.opcode);
        // Similarly, SET instructions do not have a register operand
        let has_operand_1_register = has_operand_1_register(self.opcode);

        // If there is no destination register, the destination_register field is left blank
        if has_destination_register {
            self.destination_register = get_register(instruction_text, 1)?;
        }

        // If there is no register operand, the operand_1_register field is left blank
        if has_operand_1_register {
            self.operand_1_register = get_register(instruction_text, 2)?
        }

        // All I-Format instructions are guaranteed to have an immediate operand
        self.operand_2_immediate = get_immediate(instruction_text)?;

        Ok(())
    }

    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_string = String::new();

        // Append the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnemonic) => mnemonic,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Append the destination register
        if has_destination_register(self.opcode) {
            instruction_string.push_str(&format_register(self.destination_register)?);
            instruction_string.push(' ');
        }

        // Append the register operand
        if has_operand_1_register(self.opcode) {
            instruction_string.push_str(&format_register(self.operand_1_register)?);
            instruction_string.push(' ');
        }

        // Append the immediate operand
        instruction_string.push_str(&format_immediate(self.operand_2_immediate));

        instruction_string.push('\n');

        Ok(instruction_string)
    }

    fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.destination_register as u32) << 20
            | (self.operand_1_register as u32) << 16
            | (self.operand_2_immediate as u32)
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.destination_register = extract_register(encoded_instruction, 0);
        self.operand_1_register = extract_register(encoded_instruction, 1);
        self.operand_2_immediate = extract_immediate(encoded_instruction);
    }
}

// See trait for method descriptions
impl Instruction for JTypeInstruction {
    fn assemble(&mut self, instruction_text: &str, symbol_table: &SymbolTable) -> Result<()> {
        // HALT instructions do not have a destination label
        let has_jump_label = has_jump_label(self.opcode);

        // Skip address resolution for instructions with no destination label
        if has_jump_label {
            let label = instruction_text.without_first_word();

            // Get the destination address of a given label name, if it is not a HALT
            self.destination_memory_address = match symbol_table.find_address(label.trim()) {
                Some(address) => address,
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

        // Append the mnemonic
        instruction_string.push_str(match get_mnemonic(self.opcode) {
            Some(mnemonic) => mnemonic,
            None => {
                return Err(OpcodeParseError::UnknownOpcode)
                    .context("Invalid instruction found in the machine code file.")
            }
        });

        instruction_string.push(' ');

        // Append the jump label
        if has_jump_label(self.opcode) {
            let label = match symbol_table.find_name(self.destination_memory_address) {
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
        (self.opcode as u32) << 24 | (self.destination_memory_address as u32)
    }

    fn decode(&mut self, encoded_instruction: u32) {
        self.opcode = extract_opcode(encoded_instruction);
        self.destination_memory_address = extract_address(encoded_instruction);
    }
}
