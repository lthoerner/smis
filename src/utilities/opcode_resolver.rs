use super::instruction::Instruction;


// Opcode constants
pub const OP_SET: u8                = 0x01;
pub const OP_COPY: u8               = 0x02;

pub const OP_ADD: u8                = 0x03;
pub const OP_SUBTRACT: u8           = 0x04;
pub const OP_MULTIPLY: u8           = 0x05;
pub const OP_DIVIDE: u8             = 0x06;
pub const OP_MODULO: u8             = 0x07;

pub const OP_COMPARE: u8            = 0x08;

pub const OP_SHIFT_LEFT: u8         = 0x09;
pub const OP_SHIFT_RIGHT: u8        = 0x0A;

pub const OP_AND: u8                = 0x0B;
pub const OP_OR: u8                 = 0x0C;
pub const OP_XOR: u8                = 0x0D;
pub const OP_NAND: u8               = 0x0E;
pub const OP_NOR: u8                = 0x0F;
pub const OP_NOT: u8                = 0x10;

pub const OP_ADD_IMM: u8            = 0x11;
pub const OP_SUBTRACT_IMM: u8       = 0x12;
pub const OP_MULTIPLY_IMM: u8       = 0x13;
pub const OP_DIVIDE_IMM: u8         = 0x14;
pub const OP_MODULO_IMM: u8         = 0x15;

pub const OP_COMPARE_IMM: u8        = 0x16;

pub const OP_SHIFT_LEFT_IMM: u8     = 0x17;
pub const OP_SHIFT_RIGHT_IMM: u8    = 0x18;

pub const OP_AND_IMM: u8            = 0x19;
pub const OP_OR_IMM: u8             = 0x1A;
pub const OP_XOR_IMM: u8            = 0x1B;
pub const OP_NAND_IMM: u8           = 0x1C;
pub const OP_NOR_IMM: u8            = 0x1D;

pub const OP_LOAD: u8               = 0x1E;
pub const OP_STORE: u8              = 0x1F;

pub const OP_JUMP: u8               = 0x20;
pub const OP_JUMP_IF_ZERO: u8       = 0x21;
pub const OP_JUMP_IF_NOTZERO: u8    = 0x22;
pub const OP_JUMP_LINK: u8          = 0x23;

pub const OP_HALT: u8               = 0x24;


// Gets the associated mnemonic for a given opcode
pub fn get_mnemonic(opcode: u8) -> Option<&'static str> {
    Some(match opcode { 
        OP_SET              => "SET",
        OP_COPY             => "COPY",
        OP_ADD              => "ADD",
        OP_SUBTRACT         => "SUBTRACT",
        OP_MULTIPLY         => "MULTIPLY",
        OP_DIVIDE           => "DIVIDE",
        OP_MODULO           => "MODULO",
        OP_COMPARE          => "COMPARE",
        OP_SHIFT_LEFT       => "SHIFT-LEFT",
        OP_SHIFT_RIGHT      => "SHIFT-RIGHT",
        OP_AND              => "AND",
        OP_OR               => "OR",
        OP_XOR              => "XOR",
        OP_NAND             => "NAND",
        OP_NOR              => "NOR",
        OP_NOT              => "NOT",
        OP_ADD_IMM          => "ADD-IMM",
        OP_SUBTRACT_IMM     => "SUBTRACT-IMM",
        OP_MULTIPLY_IMM     => "MULTIPLY-IMM",
        OP_DIVIDE_IMM       => "DIVIDE-IMM",
        OP_MODULO_IMM       => "MODULO-IMM",
        OP_COMPARE_IMM      => "COMPARE-IMM",
        OP_SHIFT_LEFT_IMM   => "SHIFT-LEFT-IMM",
        OP_SHIFT_RIGHT_IMM  => "SHIFT-RIGHT-IMM",
        OP_AND_IMM          => "AND-IMM",
        OP_OR_IMM           => "OR-IMM",
        OP_XOR_IMM          => "XOR-IMM",
        OP_NAND_IMM         => "NAND-IMM",
        OP_NOR_IMM          => "NOR-IMM",
        OP_LOAD             => "LOAD",
        OP_STORE            => "STORE",
        OP_JUMP             => "JUMP",
        OP_JUMP_IF_ZERO     => "JUMP-IF-ZERO",
        OP_JUMP_IF_NOTZERO  => "JUMP-IF-NOTZERO",
        OP_JUMP_LINK        => "JUMP-LINK",
        OP_HALT             => "HALT",
        _ => return None
    })
}

// Gets the associated opcode for a given mnemonic
pub fn get_opcode(mnemonic: &str) -> Option<u8> {
    Some(match mnemonic.to_lowercase().as_str() { 
        "set"               => OP_SET,
        "copy"              => OP_COPY,
        "add"               => OP_ADD,
        "subtract"          => OP_SUBTRACT,
        "multiply"          => OP_MULTIPLY,
        "divide"            => OP_DIVIDE,
        "modulo"            => OP_MODULO,
        "compare"           => OP_COMPARE,
        "shift-left"        => OP_SHIFT_LEFT,
        "shift-right"       => OP_SHIFT_RIGHT,
        "and"               => OP_AND,
        "or"                => OP_OR,
        "xor"               => OP_XOR,
        "nand"              => OP_NAND,
        "nor"               => OP_NOR,
        "not"               => OP_NOT,
        "add-imm"           => OP_ADD_IMM,
        "subtract-imm"      => OP_SUBTRACT_IMM,
        "multiply-imm"      => OP_MULTIPLY_IMM,
        "divide-imm"        => OP_DIVIDE_IMM,
        "modulo-imm"        => OP_MODULO_IMM,
        "compare-imm"       => OP_COMPARE_IMM,
        "shift-left-imm"    => OP_SHIFT_LEFT_IMM,
        "shift-right-imm"   => OP_SHIFT_RIGHT_IMM,
        "and-imm"           => OP_AND_IMM,
        "or-imm"            => OP_OR_IMM,
        "xor-imm"           => OP_XOR_IMM,
        "nand-imm"          => OP_NAND_IMM,
        "nor-imm"           => OP_NOR_IMM,
        "load"              => OP_LOAD,
        "store"             => OP_STORE,
        "jump"              => OP_JUMP,
        "jump-if-zero"      => OP_JUMP_IF_ZERO,
        "jump-if-notzero"   => OP_JUMP_IF_NOTZERO,
        "jump-link"         => OP_JUMP_LINK,
        "halt"              => OP_HALT,
        _ => return None
    })
}

// Uses the given opcode to return an enum with fields based on the instruction format
pub fn get_instruction_format(opcode: u8) -> Option<Instruction> {
    Some(match opcode { 
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
            => Instruction::RFormat {
                opcode,
                r_dest: 0,
                r_op1: 0,
                r_op2: 0
            },
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
            => Instruction::IFormat {
                opcode,
                r_dest: 0,
                r_op1: 0,
                i_op2: 0
            },
        OP_JUMP
        | OP_JUMP_IF_ZERO
        | OP_JUMP_IF_NOTZERO
        | OP_JUMP_LINK
        | OP_HALT
            => Instruction::JFormat {
                opcode,
                dest_addr: 0
            },
        _ => return None
    })
}