use super::errors::*;
use crate::assembler::*;
use crate::disassembler::*;
use crate::utilities::{opcodes::*, string_methods::SmisString, symbol_table::SymbolTable};
use anyhow::{Context, Result};

pub trait Instruction<'a>:
    TryFrom<(&'a str, &'a SymbolTable), Error = anyhow::Error>
    + TryFrom<u32, Error = anyhow::Error>
    + Into<u32>
{
    // Assembles an Instruction from a string (alternate syntax for TryFrom<(&str, &SymbolTable)>)
    fn assemble(instruction_text: &'a str, symbol_table: &'a SymbolTable) -> Result<Self> {
        Self::try_from((instruction_text, symbol_table))
            .context("Encountered invalid or malformed instruction.")
            .context(format!("At: '{}'", instruction_text))
    }

    // Disassembles an Instruction into a string
    fn disassemble(&self, symbol_table: &SymbolTable) -> Result<String>;

    // Encodes an Instruction into a u32 (alternate syntax for Into<u32>)
    fn encode(self) -> u32 {
        self.into()
    }

    // Decodes an Instruction from a u32 (alternate syntax for TryFrom<u32>)
    fn decode(encoded_instruction: u32) -> Result<Self> {
        Self::try_from(encoded_instruction)
    }
}

pub enum InstructionContainer {
    R(RTypeInstruction),
    I(ITypeInstruction),
    J(JTypeInstruction),
}

// Passthrough implementations for InstructionContainer variants
// See trait for method descriptions
impl<'a> Instruction<'a> for InstructionContainer {
    fn disassemble(&self, symbol_table: &SymbolTable) -> Result<String> {
        match self {
            InstructionContainer::R(r_type_instruction) => {
                r_type_instruction.disassemble(symbol_table)
            }
            InstructionContainer::I(i_type_instruction) => {
                i_type_instruction.disassemble(symbol_table)
            }
            InstructionContainer::J(j_type_instruction) => {
                j_type_instruction.disassemble(symbol_table)
            }
        }
    }
}

impl<'a> TryFrom<(&'a str, &'a SymbolTable)> for InstructionContainer {
    type Error = anyhow::Error;

    fn try_from((instruction_text, symbol_table): (&'a str, &'a SymbolTable)) -> Result<Self> {
        let opcode = get_opcode_from_mnemonic(instruction_text)?;
        let encoding_format = EncodingFormat::from(opcode);

        // Create an empty instruction container
        let instruction = match encoding_format {
            EncodingFormat::R => InstructionContainer::R(RTypeInstruction::try_from((
                instruction_text,
                symbol_table,
            ))?),
            EncodingFormat::I => InstructionContainer::I(ITypeInstruction::try_from((
                instruction_text,
                symbol_table,
            ))?),
            EncodingFormat::J => InstructionContainer::J(JTypeInstruction::try_from((
                instruction_text,
                symbol_table,
            ))?),
        };

        Ok(instruction)
    }
}

impl TryFrom<u32> for InstructionContainer {
    type Error = anyhow::Error;

    fn try_from(encoded_instruction: u32) -> Result<Self> {
        // TODO: Error handle
        let opcode = extract_opcode(encoded_instruction).unwrap();

        let instruction = match opcode.into() {
            EncodingFormat::R => {
                InstructionContainer::R(RTypeInstruction::try_from(encoded_instruction)?)
            }
            EncodingFormat::I => {
                InstructionContainer::I(ITypeInstruction::try_from(encoded_instruction)?)
            }
            EncodingFormat::J => {
                InstructionContainer::J(JTypeInstruction::try_from(encoded_instruction)?)
            }
        };

        Ok(instruction)
    }
}

impl From<InstructionContainer> for u32 {
    fn from(instruction: InstructionContainer) -> Self {
        match instruction {
            InstructionContainer::R(r_type_instruction) => r_type_instruction.into(),
            InstructionContainer::I(i_type_instruction) => i_type_instruction.into(),
            InstructionContainer::J(j_type_instruction) => j_type_instruction.into(),
        }
    }
}

// Instruction format structs
#[derive(Debug)]
pub struct RTypeInstruction {
    pub opcode: Opcode,
    pub destination_register: Option<u8>,
    pub operand_1_register: Option<u8>,
    pub operand_2_register: Option<u8>,
}

#[derive(Debug)]
pub struct ITypeInstruction {
    pub opcode: Opcode,
    pub destination_register: Option<u8>,
    pub operand_1_register: Option<u8>,
    pub operand_2_immediate: u16,
}

#[derive(Debug)]
pub struct JTypeInstruction {
    pub opcode: Opcode,
    pub jump_memory_address: Option<u16>,
    pub jump_register: Option<u8>,
}

// See trait for method descriptions
impl<'a> Instruction<'a> for RTypeInstruction {
    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_components = Vec::new();

        // Append the mnemonic
        instruction_components.push(self.opcode.to_string());

        // Append the destination register
        if let Some(destination_register) = self.destination_register {
            instruction_components.push(format_register(destination_register)?);
        }

        // Append the first operand register
        if let Some(operand_1_register) = self.operand_1_register {
            instruction_components.push(format_register(operand_1_register)?);
        }

        // Append the second operand register
        if let Some(operand_2_register) = self.operand_2_register {
            instruction_components.push(format_register(operand_2_register)?);
        }

        Ok(instruction_components.join(" "))
    }
}

impl<'a> TryFrom<(&'a str, &'a SymbolTable)> for RTypeInstruction {
    type Error = anyhow::Error;

    fn try_from((instruction_text, _symbol_table): (&'a str, &'a SymbolTable)) -> Result<Self> {
        let opcode = get_opcode_from_mnemonic(instruction_text)?;

        let has_destination_register = should_have_destination_register(&opcode);

        // In the case of a missing destination register,
        // all the operand words are shifted over to the left
        let missing_destination_index_adjustment = !has_destination_register as usize;

        // COMPARE instructions do not have an destination register
        let destination_register = has_destination_register
            .then(|| get_register(instruction_text, 1))
            .transpose()?;

        // PRINT instructions only use the destination register, which is called the
        // "target register" in context of a PRINT instruction
        let operand_1_register = should_have_operand_1_register(&opcode)
            .then(|| get_register(instruction_text, 2 - missing_destination_index_adjustment))
            .transpose()?;

        // Similarly, NOT and COPY instructions do not have a second operand register
        let operand_2_register = should_have_operand_2_register(&opcode)
            .then(|| get_register(instruction_text, 3 - missing_destination_index_adjustment))
            .transpose()?;

        Ok(Self {
            opcode,
            destination_register,
            operand_1_register,
            operand_2_register,
        })
    }
}

impl TryFrom<u32> for RTypeInstruction {
    type Error = anyhow::Error;

    fn try_from(encoded_instruction: u32) -> Result<Self> {
        // * This is only called if the instruction has been verified both
        // * to have a valid opcode and to be an R-Type instruction
        let opcode = extract_opcode(encoded_instruction).unwrap();
        assert_eq!(EncodingFormat::R, opcode.clone().into());

        let destination_register = should_have_destination_register(&opcode)
            .then(|| extract_register(encoded_instruction, 0));

        let operand_1_register = should_have_operand_1_register(&opcode)
            .then(|| extract_register(encoded_instruction, 1));

        let operand_2_register = should_have_operand_2_register(&opcode)
            .then(|| extract_register(encoded_instruction, 2));

        Ok(Self {
            opcode,
            destination_register,
            operand_1_register,
            operand_2_register,
        })
    }
}

impl From<RTypeInstruction> for u32 {
    fn from(instruction: RTypeInstruction) -> Self {
        let opcode = instruction.opcode.as_u8() as u32;
        let destination_register = instruction.destination_register.unwrap_or_default() as u32;
        let operand_1_register = instruction.operand_1_register.unwrap_or_default() as u32;
        let operand_2_register = instruction.operand_2_register.unwrap_or_default() as u32;

        opcode << 24
            | destination_register << 20
            | operand_1_register << 16
            | operand_2_register << 12
    }
}

// See trait for method descriptions
impl<'a> Instruction<'a> for ITypeInstruction {
    fn disassemble(&self, _symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_components = Vec::new();

        // Append the mnemonic
        instruction_components.push(self.opcode.to_string());

        // Append the destination register
        if let Some(destination_register) = self.destination_register {
            instruction_components.push(format_register(destination_register)?);
        }

        // Append the register operand
        if let Some(operand_1_register) = self.operand_1_register {
            instruction_components.push(format_register(operand_1_register)?);
        }

        // Append the immediate operand
        instruction_components.push(format_immediate(self.operand_2_immediate));

        Ok(instruction_components.join(" "))
    }
}

impl<'a> TryFrom<(&'a str, &'a SymbolTable)> for ITypeInstruction {
    type Error = anyhow::Error;

    fn try_from((instruction_text, _symbol_table): (&'a str, &'a SymbolTable)) -> Result<Self> {
        let opcode = get_opcode_from_mnemonic(instruction_text)?;

        // COMPARE-IMM instructions do not have a destination register
        let destination_register = should_have_destination_register(&opcode)
            .then(|| get_register(instruction_text, 1))
            .transpose()?;

        let no_destination_index_adjustment = destination_register.is_none() as usize;

        // Similarly, SET instructions do not have a register operand
        let operand_1_register = should_have_operand_1_register(&opcode)
            .then(|| get_register(instruction_text, 2 - no_destination_index_adjustment))
            .transpose()?;

        // All I-Format instructions are guaranteed to have an immediate operand
        let operand_2_immediate = get_immediate(instruction_text)?;

        Ok(Self {
            opcode,
            destination_register,
            operand_1_register,
            operand_2_immediate,
        })
    }
}

impl TryFrom<u32> for ITypeInstruction {
    type Error = anyhow::Error;

    fn try_from(encoded_instruction: u32) -> Result<Self> {
        // * This is only called if the instruction has been verified both
        // * to have a valid opcode and to be an I-Type instruction
        let opcode = extract_opcode(encoded_instruction).unwrap();
        assert_eq!(EncodingFormat::I, opcode.clone().into());

        let destination_register = should_have_destination_register(&opcode)
            .then(|| extract_register(encoded_instruction, 0));

        let operand_1_register = should_have_operand_1_register(&opcode)
            .then(|| extract_register(encoded_instruction, 1));

        // All I-Format instructions are guaranteed to have an immediate operand
        let operand_2_immediate = extract_immediate(encoded_instruction);

        Ok(Self {
            opcode,
            destination_register,
            operand_1_register,
            operand_2_immediate,
        })
    }
}

impl From<ITypeInstruction> for u32 {
    fn from(instruction: ITypeInstruction) -> Self {
        let opcode = instruction.opcode.as_u8() as u32;
        let destination_register = instruction.destination_register.unwrap_or_default() as u32;
        let operand_1_register = instruction.operand_1_register.unwrap_or_default() as u32;
        let operand_2_immediate = instruction.operand_2_immediate as u32;

        opcode << 24 | destination_register << 20 | operand_1_register << 16 | operand_2_immediate
    }
}

// See trait for method descriptions
impl<'a> Instruction<'a> for JTypeInstruction {
    fn disassemble(&self, symbol_table: &SymbolTable) -> Result<String> {
        let mut instruction_components = Vec::new();

        // Append the mnemonic
        instruction_components.push(self.opcode.to_string());

        // Append the jump label
        if let Some(destination_memory_address) = self.jump_memory_address {
            let label = match symbol_table.find_name(destination_memory_address) {
                Some(label) => label,
                None => {
                    return Err(SymbolTableError::LabelNotFound)
                        .context("[INTERNAL ERROR] Label not found in symbol table.")
                }
            };

            instruction_components.push(label);
        }

        Ok(instruction_components.join(" "))
    }
}

impl<'a> TryFrom<(&'a str, &'a SymbolTable)> for JTypeInstruction {
    type Error = anyhow::Error;

    fn try_from((instruction_text, symbol_table): (&'a str, &'a SymbolTable)) -> Result<Self> {
        let opcode = get_opcode_from_mnemonic(instruction_text)?;

        let mut jump_memory_address = None;

        // Skip address resolution for instructions with no jump label
        // HALT and JUMP-REGISTER instructions do not have a jump label
        if should_have_jump_label(&opcode) {
            let label = instruction_text.without_first_word();

            // Get the jump address of a given label name
            let Some(address) = symbol_table.find_address(label.trim()) else {
                return Err(SymbolTableError::LabelNotFound)
                    .context("Label not found in symbol table.")
                    .context(format!("At: '{}'", label))
            };

            jump_memory_address = Some(address);
        }

        // JUMP-REGISTER instructions have a jump register
        let jump_register = should_have_jump_register(&opcode)
            .then(|| get_register(instruction_text, 1))
            .transpose()?;

        Ok(Self {
            opcode,
            jump_memory_address,
            jump_register,
        })
    }
}

impl TryFrom<u32> for JTypeInstruction {
    type Error = anyhow::Error;

    fn try_from(encoded_instruction: u32) -> Result<Self> {
        // * This is only called if the instruction has been verified both
        // * to have a valid opcode and to be an J-Type instruction
        let opcode = extract_opcode(encoded_instruction).unwrap();
        assert_eq!(EncodingFormat::J, opcode.clone().into());

        let jump_memory_address =
            should_have_jump_label(&opcode).then(|| extract_address(encoded_instruction));

        let jump_register =
            should_have_jump_register(&opcode).then(|| extract_register(encoded_instruction, 0));

        Ok(Self {
            opcode,
            jump_memory_address,
            jump_register,
        })
    }
}

impl From<JTypeInstruction> for u32 {
    fn from(instruction: JTypeInstruction) -> Self {
        let opcode = instruction.opcode.as_u8() as u32;
        let jump_memory_address = instruction.jump_memory_address.unwrap_or_default() as u32;
        let jump_register = instruction.jump_register.unwrap_or_default() as u32;

        opcode << 24 | jump_register << 20 | jump_memory_address
    }
}
