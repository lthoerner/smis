use super::instructions::{ITypeInstruction, Instruction, JTypeInstruction, RTypeInstruction};

// Opcode constants
pub const OP_SET: u8 = 0x01;
pub const OP_COPY: u8 = 0x02;

pub const OP_ADD: u8 = 0x03;
pub const OP_SUBTRACT: u8 = 0x04;
pub const OP_MULTIPLY: u8 = 0x05;
pub const OP_DIVIDE: u8 = 0x06;
pub const OP_MODULO: u8 = 0x07;

pub const OP_COMPARE: u8 = 0x08;

pub const OP_SHIFT_LEFT: u8 = 0x09;
pub const OP_SHIFT_RIGHT: u8 = 0x0A;

pub const OP_AND: u8 = 0x0B;
pub const OP_OR: u8 = 0x0C;
pub const OP_XOR: u8 = 0x0D;
pub const OP_NAND: u8 = 0x0E;
pub const OP_NOR: u8 = 0x0F;
pub const OP_NOT: u8 = 0x10;

pub const OP_ADD_IMM: u8 = 0x11;
pub const OP_SUBTRACT_IMM: u8 = 0x12;
pub const OP_MULTIPLY_IMM: u8 = 0x13;
pub const OP_DIVIDE_IMM: u8 = 0x14;
pub const OP_MODULO_IMM: u8 = 0x15;

pub const OP_COMPARE_IMM: u8 = 0x16;

pub const OP_SHIFT_LEFT_IMM: u8 = 0x17;
pub const OP_SHIFT_RIGHT_IMM: u8 = 0x18;

pub const OP_AND_IMM: u8 = 0x19;
pub const OP_OR_IMM: u8 = 0x1A;
pub const OP_XOR_IMM: u8 = 0x1B;
pub const OP_NAND_IMM: u8 = 0x1C;
pub const OP_NOR_IMM: u8 = 0x1D;

pub const OP_LOAD: u8 = 0x1E;
pub const OP_STORE: u8 = 0x1F;

pub const OP_JUMP: u8 = 0x20;
pub const OP_JUMP_IF_ZERO: u8 = 0x21;
pub const OP_JUMP_IF_NOTZERO: u8 = 0x22;
pub const OP_JUMP_LINK: u8 = 0x23;

pub const OP_HALT: u8 = 0x24;

pub const OP_PRINT: u8 = 0x25;

#[allow(dead_code)]
// Gets the associated mnemonic for a given opcode
pub fn get_mnemonic(opcode: u8) -> Option<&'static str> {
    Some(match opcode {
        OP_SET => "SET",
        OP_COPY => "COPY",
        OP_ADD => "ADD",
        OP_SUBTRACT => "SUBTRACT",
        OP_MULTIPLY => "MULTIPLY",
        OP_DIVIDE => "DIVIDE",
        OP_MODULO => "MODULO",
        OP_COMPARE => "COMPARE",
        OP_SHIFT_LEFT => "SHIFT-LEFT",
        OP_SHIFT_RIGHT => "SHIFT-RIGHT",
        OP_AND => "AND",
        OP_OR => "OR",
        OP_XOR => "XOR",
        OP_NAND => "NAND",
        OP_NOR => "NOR",
        OP_NOT => "NOT",
        OP_ADD_IMM => "ADD-IMM",
        OP_SUBTRACT_IMM => "SUBTRACT-IMM",
        OP_MULTIPLY_IMM => "MULTIPLY-IMM",
        OP_DIVIDE_IMM => "DIVIDE-IMM",
        OP_MODULO_IMM => "MODULO-IMM",
        OP_COMPARE_IMM => "COMPARE-IMM",
        OP_SHIFT_LEFT_IMM => "SHIFT-LEFT-IMM",
        OP_SHIFT_RIGHT_IMM => "SHIFT-RIGHT-IMM",
        OP_AND_IMM => "AND-IMM",
        OP_OR_IMM => "OR-IMM",
        OP_XOR_IMM => "XOR-IMM",
        OP_NAND_IMM => "NAND-IMM",
        OP_NOR_IMM => "NOR-IMM",
        OP_LOAD => "LOAD",
        OP_STORE => "STORE",
        OP_JUMP => "JUMP",
        OP_JUMP_IF_ZERO => "JUMP-IF-ZERO",
        OP_JUMP_IF_NOTZERO => "JUMP-IF-NOTZERO",
        OP_JUMP_LINK => "JUMP-LINK",
        OP_HALT => "HALT",
        OP_PRINT => "PRINT",
        _ => return None,
    })
}

// Gets the associated opcode for a given mnemonic
pub fn get_opcode(mnemonic: &str) -> Option<u8> {
    Some(match mnemonic.to_uppercase().as_str() {
        "SET" => OP_SET,
        "COPY" => OP_COPY,
        "ADD" => OP_ADD,
        "SUBTRACT" => OP_SUBTRACT,
        "MULTIPLY" => OP_MULTIPLY,
        "DIVIDE" => OP_DIVIDE,
        "MODULO" => OP_MODULO,
        "COMPARE" => OP_COMPARE,
        "SHIFT-LEFT" => OP_SHIFT_LEFT,
        "SHIFT-RIGHT" => OP_SHIFT_RIGHT,
        "AND" => OP_AND,
        "OR" => OP_OR,
        "XOR" => OP_XOR,
        "NAND" => OP_NAND,
        "NOR" => OP_NOR,
        "NOT" => OP_NOT,
        "ADD-IMM" => OP_ADD_IMM,
        "SUBTRACT-IMM" => OP_SUBTRACT_IMM,
        "MULTIPLY-IMM" => OP_MULTIPLY_IMM,
        "DIVIDE-IMM" => OP_DIVIDE_IMM,
        "MODULO-IMM" => OP_MODULO_IMM,
        "COMPARE-IMM" => OP_COMPARE_IMM,
        "SHIFT-LEFT-IMM" => OP_SHIFT_LEFT_IMM,
        "SHIFT-RIGHT-IMM" => OP_SHIFT_RIGHT_IMM,
        "AND-IMM" => OP_AND_IMM,
        "OR-IMM" => OP_OR_IMM,
        "XOR-IMM" => OP_XOR_IMM,
        "NAND-IMM" => OP_NAND_IMM,
        "NOR-IMM" => OP_NOR_IMM,
        "LOAD" => OP_LOAD,
        "STORE" => OP_STORE,
        "JUMP" => OP_JUMP,
        "JUMP-IF-ZERO" => OP_JUMP_IF_ZERO,
        "JUMP-IF-NOTZERO" => OP_JUMP_IF_NOTZERO,
        "JUMP-LINK" => OP_JUMP_LINK,
        "HALT" => OP_HALT,
        "PRINT" => OP_PRINT,
        _ => return None,
    })
}

// Uses the given opcode to return an enum with fields based on the instruction format
// TODO: This would probably be better as Into<Instruction> implementations, would be more intuitive
pub fn get_instruction(opcode: u8) -> Option<Instruction> {
    Some(if is_r_type(opcode) {
        Instruction::R(RTypeInstruction::new(opcode))
    } else if is_i_type(opcode) {
        Instruction::I(ITypeInstruction::new(opcode))
    } else if is_j_type(opcode) {
        Instruction::J(JTypeInstruction::new(opcode))
    } else {
        return None;
    })
}

pub fn is_r_type(opcode: u8) -> bool {
    matches!(
        opcode,
        OP_COPY
            | OP_ADD
            | OP_SUBTRACT
            | OP_MULTIPLY
            | OP_DIVIDE
            | OP_MODULO
            | OP_COMPARE
            | OP_SHIFT_LEFT
            | OP_SHIFT_RIGHT
            | OP_AND
            | OP_OR
            | OP_XOR
            | OP_NAND
            | OP_NOR
            | OP_NOT
            | OP_PRINT
    )
}

pub fn is_i_type(opcode: u8) -> bool {
    matches!(
        opcode,
        OP_SET
            | OP_ADD_IMM
            | OP_SUBTRACT_IMM
            | OP_MULTIPLY_IMM
            | OP_DIVIDE_IMM
            | OP_MODULO_IMM
            | OP_COMPARE_IMM
            | OP_SHIFT_LEFT_IMM
            | OP_SHIFT_RIGHT_IMM
            | OP_AND_IMM
            | OP_OR_IMM
            | OP_XOR_IMM
            | OP_NAND_IMM
            | OP_NOR_IMM
            | OP_LOAD
            | OP_STORE
    )
}

pub fn is_j_type(opcode: u8) -> bool {
    matches!(
        opcode,
        OP_JUMP | OP_JUMP_IF_ZERO | OP_JUMP_IF_NOTZERO | OP_JUMP_LINK | OP_HALT
    )
}

pub fn has_destination_register(opcode: u8) -> bool {
    !matches!(opcode, OP_COMPARE | OP_COMPARE_IMM)
}

pub fn has_operand_1_register(opcode: u8) -> bool {
    !matches!(opcode, OP_SET | OP_PRINT)
}

pub fn has_operand_2_register(opcode: u8) -> bool {
    !matches!(opcode, OP_COPY | OP_NOT | OP_PRINT)
}

pub fn has_jump_label(opcode: u8) -> bool {
    !matches!(opcode, OP_HALT)
}

pub fn extract_opcode(instruction: u32) -> u8 {
    ((instruction & 0xFF000000) >> 24) as u8
}
