// TODO: Turn these off when no longer needed
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]


use std::path::Path;
use utilities::user_messages;


mod assembler;
mod disassembler;
mod emulator;
mod utilities;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        panic!("Incorrect number of arguments!\n{}", user_messages::USAGE_ERROR);
    }

    let target = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        panic!("Input file does not exist!\n{}", user_messages::USAGE_ERROR);
    }

    match target.as_str() {
        "--assemble" | "-a" => {
            assembler::start_assembler(input_file, output_file);
            println!("File assembled successfully!")
        },
        // "--disassemble" | "-d" => disassembler::disassemble(input_file, output_file),
        _ => panic!("Invalid target \"{}\"!\n{}", target, user_messages::USAGE_ERROR)
    }
}