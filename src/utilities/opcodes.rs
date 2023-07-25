use super::errors::*;
use anyhow::{Context, Result};
use std::fmt::{Display, Formatter};

macro_rules! u8_enum {
    ($name:ident { $($variant:ident = $value:expr,)* }) => {
        #[derive(Debug, Clone)]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            fn from_u8(val: u8) -> Option<Self> {
                match val {
                    $( $value => Some(Self::$variant), )*
                    _ => None,
                }
            }

            pub fn as_u8(&self) -> u8 {
                match self {
                    $( Self::$variant => $value, )*
                }
            }
        }
    };
}

u8_enum! {
    Opcode {
        Set = 0x01,
        Copy = 0x02,
        Add = 0x03,
        Subtract = 0x04,
        Multiply = 0x05,
        Divide = 0x06,
        Modulo = 0x07,
        Compare = 0x08,
        ShiftLeft = 0x09,
        ShiftRight = 0x0A,
        And = 0x0B,
        Or = 0x0C,
        Xor = 0x0D,
        Nand = 0x0E,
        Nor = 0x0F,
        Not = 0x10,
        AddImm = 0x11,
        SubtractImm = 0x12,
        MultiplyImm = 0x13,
        DivideImm = 0x14,
        ModuloImm = 0x15,
        CompareImm = 0x16,
        ShiftLeftImm = 0x17,
        ShiftRightImm = 0x18,
        AndImm = 0x19,
        OrImm = 0x1A,
        XorImm = 0x1B,
        NandImm = 0x1C,
        NorImm = 0x1D,
        Load = 0x1E,
        Store = 0x1F,
        Jump = 0x20,
        JumpIfZero = 0x21,
        JumpIfNotZero = 0x22,
        JumpLink = 0x23,
        Halt = 0x24,
        Print = 0x25,
    }
}

#[derive(Debug, PartialEq)]
pub enum EncodingFormat {
    R,
    I,
    J,
}

impl From<Opcode> for EncodingFormat {
    fn from(opcode: Opcode) -> Self {
        use Opcode::*;
        match opcode {
            Copy | Add | Subtract | Multiply | Divide | Modulo | Compare | ShiftLeft
            | ShiftRight | And | Or | Xor | Nand | Nor | Not | Print => EncodingFormat::R,
            Set | AddImm | SubtractImm | MultiplyImm | DivideImm | ModuloImm | CompareImm
            | ShiftLeftImm | ShiftRightImm | AndImm | OrImm | XorImm | NandImm | NorImm | Load
            | Store => EncodingFormat::I,
            Jump | JumpIfZero | JumpIfNotZero | JumpLink | Halt => EncodingFormat::J,
        }
    }
}

impl TryFrom<String> for Opcode {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self> {
        let opcode = match s.to_uppercase().as_str() {
            "SET" => Opcode::Set,
            "COPY" => Opcode::Copy,
            "ADD" => Opcode::Add,
            "SUBTRACT" => Opcode::Subtract,
            "MULTIPLY" => Opcode::Multiply,
            "DIVIDE" => Opcode::Divide,
            "MODULO" => Opcode::Modulo,
            "COMPARE" => Opcode::Compare,
            "SHIFT-LEFT" => Opcode::ShiftLeft,
            "SHIFT-RIGHT" => Opcode::ShiftRight,
            "AND" => Opcode::And,
            "OR" => Opcode::Or,
            "XOR" => Opcode::Xor,
            "NAND" => Opcode::Nand,
            "NOR" => Opcode::Nor,
            "NOT" => Opcode::Not,
            "ADD-IMM" => Opcode::AddImm,
            "SUBTRACT-IMM" => Opcode::SubtractImm,
            "MULTIPLY-IMM" => Opcode::MultiplyImm,
            "DIVIDE-IMM" => Opcode::DivideImm,
            "MODULO-IMM" => Opcode::ModuloImm,
            "COMPARE-IMM" => Opcode::CompareImm,
            "SHIFT-LEFT-IMM" => Opcode::ShiftLeftImm,
            "SHIFT-RIGHT-IMM" => Opcode::ShiftRightImm,
            "AND-IMM" => Opcode::AndImm,
            "OR-IMM" => Opcode::OrImm,
            "XOR-IMM" => Opcode::XorImm,
            "NAND-IMM" => Opcode::NandImm,
            "NOR-IMM" => Opcode::NorImm,
            "LOAD" => Opcode::Load,
            "STORE" => Opcode::Store,
            "JUMP" => Opcode::Jump,
            "JUMP-IF-ZERO" => Opcode::JumpIfZero,
            "JUMP-IF-NOTZERO" => Opcode::JumpIfNotZero,
            "JUMP-LINK" => Opcode::JumpLink,
            "HALT" => Opcode::Halt,
            "PRINT" => Opcode::Print,
            _ => {
                return Err(MnemonicParseError::UnknownMnemonic)
                    .context("Encountered invalid or malformed mnemonic.")
            }
        };

        Ok(opcode)
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Opcode::*;
        let mnemonic = match self {
            Set => "SET",
            Copy => "COPY",
            Add => "ADD",
            Subtract => "SUBTRACT",
            Multiply => "MULTIPLY",
            Divide => "DIVIDE",
            Modulo => "MODULO",
            Compare => "COMPARE",
            ShiftLeft => "SHIFT-LEFT",
            ShiftRight => "SHIFT-RIGHT",
            And => "AND",
            Or => "OR",
            Xor => "XOR",
            Nand => "NAND",
            Nor => "NOR",
            Not => "NOT",
            AddImm => "ADD-IMM",
            SubtractImm => "SUBTRACT-IMM",
            MultiplyImm => "MULTIPLY-IMM",
            DivideImm => "DIVIDE-IMM",
            ModuloImm => "MODULO-IMM",
            CompareImm => "COMPARE-IMM",
            ShiftLeftImm => "SHIFT-LEFT-IMM",
            ShiftRightImm => "SHIFT-RIGHT-IMM",
            AndImm => "AND-IMM",
            OrImm => "OR-IMM",
            XorImm => "XOR-IMM",
            NandImm => "NAND-IMM",
            NorImm => "NOR-IMM",
            Load => "LOAD",
            Store => "STORE",
            Jump => "JUMP",
            JumpIfZero => "JUMP-IF-ZERO",
            JumpIfNotZero => "JUMP-IF-NOTZERO",
            JumpLink => "JUMP-LINK",
            Halt => "HALT",
            Print => "PRINT",
        };

        write!(f, "{}", mnemonic)
    }
}

pub fn should_have_destination_register(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::Compare | Opcode::CompareImm)
}

pub fn should_have_operand_1_register(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::Set | Opcode::Print)
}

pub fn should_have_operand_2_register(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::Copy | Opcode::Not | Opcode::Print)
}

pub fn should_have_jump_label(opcode: &Opcode) -> bool {
    !matches!(opcode, Opcode::Halt)
}

pub fn extract_opcode(instruction: u32) -> Option<Opcode> {
    Opcode::from_u8(((instruction & 0xFF000000) >> 24) as u8)
}
