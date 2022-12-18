// Gets a generic label name based on the given label number
pub fn generate_label_name(label_number: u16) -> String {
    format!("label_{}", label_number)
}

// Gets the first 8 bits of the instruction (the opcode)
pub fn extract_opcode(instruction: u32) -> u8 {
    ((instruction & 0xFF000000) >> 24) as u8
}

// Gets an indexed register operand from the instruction
// Assumes that the index is between 0-2 (inclusive), because using Result<u8>
// would lead to way more complexity with no real benefit
pub fn extract_register(instruction: u32, index: usize) -> u8 {
    // Grab the register from the instruction by masking out a 4-bit section (shifted by the index)
    (instruction & (0x00F00000 >> (index * 4))) as u8
}

// Gets the immediate operand from the instruction
pub fn extract_immediate(instruction: u32) -> u16 {
    (instruction & 0x0000FFFF) as u16
}

// Gets the label address from the instruction
// Functionally the same as extract_immediate() but included for clarity
pub fn extract_address(instruction: u32) -> u16 {
    extract_immediate(instruction)
}
