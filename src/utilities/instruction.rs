pub struct Instruction {
    format: InstructionFormat,
    opcode: u8,
    byte_code: u32
}

pub enum InstructionFormat {
    R,
    I,
    J
}