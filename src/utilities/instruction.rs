// Represents an instruction of a given format; holds all fields separately for easier parsing
pub enum Instruction {
    RFormat {
        opcode: u8,
        r_dest: u8,
        r_op1: u8,
        r_op2: u8
    },
    
    IFormat {
        opcode: u8,
        r_dest: u8,
        r_op1: u8,
        i_op2: u16
    },

    JFormat {
        opcode: u8,
        dest_addr: u16
    }
}

impl Instruction {
    // Transforms an Instruction into its u32 representation
    pub fn encode(self) -> u32 {
        match self {
            Self::RFormat { opcode, r_dest, r_op1, r_op2 } => (opcode as u32) << 24 | (r_dest as u32) << 20 | (r_op1 as u32) << 16 | (r_op2 as u32) << 12,
            Self::IFormat { opcode, r_dest, r_op1, i_op2 } => (opcode as u32) << 24 | (r_dest as u32) << 20 | (r_op1 as u32) << 16 | (i_op2 as u32),
            Self::JFormat { opcode, dest_addr } => (opcode as u32) << 24 | (dest_addr as u32)
        }
    }
}