// TODO: Turn these off when no longer needed
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]


use std::path::Path;


mod assembler;
mod disassembler;
mod emulator;
mod utilities;


const USAGE_ERROR: &str = "Usage: <target> <input_file> <output_file>
Target must be either --assemble [-a] or --disassemble [-d].";


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        panic!("Incorrect number of arguments!\n{}", USAGE_ERROR);
    }

    let target = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        panic!("Input file does not exist!\n{}", USAGE_ERROR);
    }

    if !Path::new(output_file).exists() {
        panic!("Output file does not exits!\n{}", USAGE_ERROR);
    }

    match target.as_str() {
        "--assemble" | "-a" => match assembler::assemble(input_file, output_file) {
            Ok(()) => println!("File assembled successfully."),
            Err(_) => println!("File failed to assemble.")
        },
        // "--disassemble" | "-d" => disassembler::disassemble(input_file, output_file),
        _ => panic!("Invalid target \"{}\"!\n{}", target, USAGE_ERROR)
    }
}