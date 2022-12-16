pub enum InstructionContainer {
    RFormat(RFormat),
    IFormat(IFormat),
    JFormat(JFormat),
}

#[derive(Default)]
pub struct RFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub r_op2: u8,
}

impl RFormat {
    pub fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.r_dest as u32) << 20
            | (self.r_op1 as u32) << 16
            | (self.r_op2 as u32) << 12
    }
}

#[derive(Default)]
pub struct IFormat {
    pub opcode: u8,
    pub r_dest: u8,
    pub r_op1: u8,
    pub i_op2: u16,
}

impl IFormat {
    pub fn encode(&self) -> u32 {
        (self.opcode as u32) << 24
            | (self.r_dest as u32) << 20
            | (self.r_op1 as u32) << 16
            | (self.i_op2 as u32)
    }
}

#[derive(Default)]
pub struct JFormat {
    pub opcode: u8,
    pub dest_addr: u16,
}

impl JFormat {
    pub fn encode(&self) -> u32 {
        (self.opcode as u32) << 24 | (self.dest_addr as u32)
    }
}
