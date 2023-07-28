#![allow(dead_code)]
use crate::utilities::{
    errors::*,
    instructions::{ITypeInstruction, JTypeInstruction, RTypeInstruction},
    instructions::{Instruction, InstructionContainer},
    opcodes::*,
};
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{stdout, BufReader, ErrorKind, Read, Seek, Write};

struct Emulator {
    // The 16 general-purpose registers
    registers: [u16; 16],
    // The main memory of the emulator
    memory: [u16; u16::MAX as usize],
    // The address of the next instruction to be executed
    program_counter: u16,
    // The dedicated register containing the current instruction being executed
    instruction_register: u32,
    // Whether the result of the last operation was zero
    zero_flag: bool,
    // Whether the result of the last operation was negative
    sign_flag: bool,
    // Whether the emulator will exit before executing the next instruction
    should_exit: bool,
}

impl Emulator {
    fn new() -> Self {
        Emulator {
            registers: [0; 16],
            memory: [0; u16::MAX as usize],
            program_counter: 0,
            instruction_register: 0,
            zero_flag: false,
            sign_flag: false,
            should_exit: false,
        }
    }

    fn load_program(&mut self, binary_filename: &str) -> Result<()> {
        // Ensure the input and output files have the correct extensions
        if !binary_filename.ends_with(".bin") {
            return Err(FileHandlerError::InvalidExtension)
                .context("Machine code file must have a .bin extension.");
        }

        // Open the machine code file
        let Ok(binary_file) = File::options()
            .read(true)
            .create(false)
            .open(binary_filename)
        else {
            return Err(FileHandlerError::FileOpenFailed)
                .context("Couldn't open the machine code file. Make sure the file exists and is in the necessary directory.");
        };

        let mut reader = BufReader::new(binary_file);
        reader
            .rewind()
            .map_err(|_| FileHandlerError::FileRewindFailed)
            .context(
                "[INTERNAL ERROR] Couldn't rewind the machine code file to load the program.",
            )?;

        // TODO: This can be deduplicated with the disassembler
        let mut instruction_store_address = 0;
        // Read each instruction from the file
        loop {
            // Stores the current instruction
            let mut current_instruction = [0; 4];

            // Read 4-byte chunks of the file (instructions)
            match reader.read_exact(&mut current_instruction) {
                Ok(_) => (),
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof => break,
                    _ => return Err(FileHandlerError::FileReadFailed).context(
                        "The provided machine code file contains malformed instructions and therefore is invalid or corrupted.",
                    ),
                },
            }

            // Take the bytes and put them in a single u32, converting from network byte order
            // if needed, then add the instruction to the program
            let instruction = u32::from_be_bytes(current_instruction);
            let instruction_half_1 = (instruction >> 16) as u16;
            let instruction_half_2 = (instruction & 0x0000FFFF) as u16;

            self.memory[instruction_store_address] = instruction_half_1;
            self.memory[instruction_store_address + 1] = instruction_half_2;

            instruction_store_address += 2;
        }

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        loop {
            // If a HALT instruction has been executed, exit the program
            if self.should_exit {
                return Ok(());
            }

            self.fetch();

            // If the instruction register is empty, the program has ended without an explicit
            // HALT instruction; this isn't necessarily an error, but it is unadvisable to
            // rely on this behavior because the memory could have been overwritten
            if self.instruction_register == 0x00000000 {
                return Ok(());
            }

            let instruction = self.decode()?;

            self.execute(instruction);
        }
    }

    // TODO: Probably need bounds checking for indexing
    fn fetch(&mut self) {
        self.instruction_register = 0;
        self.instruction_register |= (self.memory[self.program_counter as usize] as u32) << 16;
        self.instruction_register |= self.memory[(self.program_counter + 1) as usize] as u32;

        self.program_counter += 2;
    }

    fn decode(&mut self) -> Result<InstructionContainer> {
        // ? Should this give more context?
        InstructionContainer::decode(self.instruction_register)
    }

    fn execute(&mut self, instruction: InstructionContainer) {
        match instruction {
            InstructionContainer::R(i) => self.execute_r_type(i),
            InstructionContainer::I(i) => self.execute_i_type(i),
            InstructionContainer::J(i) => self.execute_j_type(i),
        }
    }

    fn execute_r_type(&mut self, instruction: RTypeInstruction) {
        use Opcode::*;
        match instruction.opcode {
            Copy => self.COPY(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
            ),

            Add => self.ADD(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Subtract => self.SUBTRACT(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Multiply => self.MULTIPLY(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Divide => self.DIVIDE(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Modulo => self.MODULO(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),

            Compare => self.COMPARE(
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),

            ShiftLeft => self.SHIFT_LEFT(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            ShiftRight => self.SHIFT_RIGHT(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),

            And => self.AND(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Or => self.OR(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Xor => self.XOR(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Nand => self.NAND(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Nor => self.NOR(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_register.unwrap(),
            ),
            Not => self.NOT(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
            ),

            Print => self.PRINT(instruction.destination_register.unwrap()),

            // TODO: Actual error handling
            _ => panic!(),
        }
    }

    fn execute_i_type(&mut self, instruction: ITypeInstruction) {
        use Opcode::*;
        match instruction.opcode {
            Set => self.SET(
                instruction.destination_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            AddImm => self.ADD_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            SubtractImm => self.SUBTRACT_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            MultiplyImm => self.MULTIPLY_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            DivideImm => self.DIVIDE_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            ModuloImm => self.MODULO_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            CompareImm => self.COMPARE_IMM(
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            ShiftLeftImm => self.SHIFT_LEFT_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            ShiftRightImm => self.SHIFT_RIGHT_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            AndImm => self.AND_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            OrImm => self.OR_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            XorImm => self.XOR_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            NandImm => self.NAND_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            NorImm => self.NOR_IMM(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            Load => self.LOAD(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),
            Store => self.STORE(
                instruction.destination_register.unwrap(),
                instruction.operand_1_register.unwrap(),
                instruction.operand_2_immediate,
            ),

            _ => panic!(),
        }
    }

    fn execute_j_type(&mut self, instruction: JTypeInstruction) {
        use Opcode::*;
        match instruction.opcode {
            Jump => self.JUMP(instruction.jump_memory_address.unwrap()),
            JumpIfZero => self.JUMP_IF_ZERO(instruction.jump_memory_address.unwrap()),
            JumpIfNotZero => self.JUMP_IF_NOTZERO(instruction.jump_memory_address.unwrap()),
            JumpLink => self.JUMP_LINK(instruction.jump_memory_address.unwrap()),
            JumpRegister => self.JUMP_REGISTER(instruction.jump_register.unwrap()),

            Halt => self.HALT(),

            _ => panic!(),
        }
    }

    // TODO: Deduplicate code
    fn SET(&mut self, destination_register: u8, immediate_value: u16) {
        self.registers[destination_register as usize] = immediate_value;
    }

    fn COPY(&mut self, destination_register: u8, source_register: u8) {
        self.registers[destination_register as usize] = self.registers[source_register as usize];
    }

    // TODO: Do I need any special arithmetic methods for overflow?
    fn ADD(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            + self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn SUBTRACT(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_register: u8,
    ) {
        let result = self.registers[operand_1_register as usize]
            - self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn MULTIPLY(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_register: u8,
    ) {
        let result = self.registers[operand_1_register as usize]
            * self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn DIVIDE(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            / self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn MODULO(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            % self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn COMPARE(&mut self, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            - self.registers[operand_2_register as usize];

        self.set_flags(result);
    }

    fn SHIFT_LEFT(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_register: u8,
    ) {
        let result = self.registers[operand_1_register as usize]
            << self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn SHIFT_RIGHT(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_register: u8,
    ) {
        let result = self.registers[operand_1_register as usize]
            >> self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn AND(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            & self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn OR(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            | self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn XOR(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = self.registers[operand_1_register as usize]
            ^ self.registers[operand_2_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn NAND(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = !(self.registers[operand_1_register as usize]
            & self.registers[operand_2_register as usize]);
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn NOR(&mut self, destination_register: u8, operand_1_register: u8, operand_2_register: u8) {
        let result = !(self.registers[operand_1_register as usize]
            | self.registers[operand_2_register as usize]);
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn NOT(&mut self, destination_register: u8, operand_1_register: u8) {
        let result = !self.registers[operand_1_register as usize];
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn ADD_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] + operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn SUBTRACT_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] - operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn MULTIPLY_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] * operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn DIVIDE_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] / operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn MODULO_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] % operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn COMPARE_IMM(&mut self, operand_1_register: u8, operand_2_immediate: u16) {
        let result = self.registers[operand_1_register as usize] - operand_2_immediate;

        self.set_flags(result);
    }

    fn SHIFT_LEFT_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] << operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn SHIFT_RIGHT_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] >> operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn AND_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] & operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn OR_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] | operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn XOR_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = self.registers[operand_1_register as usize] ^ operand_2_immediate;
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn NAND_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = !(self.registers[operand_1_register as usize] & operand_2_immediate);
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn NOR_IMM(
        &mut self,
        destination_register: u8,
        operand_1_register: u8,
        operand_2_immediate: u16,
    ) {
        let result = !(self.registers[operand_1_register as usize] | operand_2_immediate);
        self.registers[destination_register as usize] = result;

        self.set_flags(result);
    }

    fn LOAD(&mut self, destination_register: u8, base_address_register: u8, offset_immediate: u16) {
        let address = self.registers[base_address_register as usize] + offset_immediate;
        self.registers[destination_register as usize] = self.memory[address as usize];
    }

    fn STORE(&mut self, source_register: u8, base_address_register: u8, offset_immediate: u16) {
        let address = self.registers[base_address_register as usize] + offset_immediate;
        self.memory[address as usize] = self.registers[source_register as usize];
    }

    fn JUMP(&mut self, address_immediate: u16) {
        self.program_counter = address_immediate;
    }

    fn JUMP_IF_ZERO(&mut self, address_immediate: u16) {
        if self.zero_flag {
            self.program_counter = address_immediate;
        }
    }

    fn JUMP_IF_NOTZERO(&mut self, address_immediate: u16) {
        if !self.zero_flag {
            self.program_counter = address_immediate;
        }
    }

    fn JUMP_LINK(&mut self, address_immediate: u16) {
        self.registers[13] = self.program_counter;
        self.program_counter = address_immediate;
    }

    fn JUMP_REGISTER(&mut self, address_register: u8) {
        self.program_counter = self.registers[address_register as usize];
    }

    fn PRINT(&mut self, target_register: u8) {
        // Get the first byte in the register and convert it to a char
        let char_to_print = (self.registers[target_register as usize] & 0xFF) as u8 as char;
        print!("{}", char_to_print);
        stdout().flush().unwrap();
    }

    fn HALT(&mut self) {
        self.should_exit = true;
    }

    fn set_flags(&mut self, result: u16) {
        self.zero_flag = result == 0;
        self.sign_flag = (result as i16) < 0;
    }
}

pub fn start_emulator(binary_filename: &str) -> Result<()> {
    let mut emulator = Emulator::new();
    emulator.load_program(binary_filename)?;
    emulator.run()
}
