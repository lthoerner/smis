// This is here because the compiler complains about the crate being called "SMIS" instead of "smis",
// despite the fact that it's an acronym and should be all caps
#![allow(non_snake_case)]

use std::path::Path;
use utilities::user_messages;

mod assembler;
mod disassembler;
mod emulator;
mod errors;
mod utilities;

fn main() {
    // let now = std::time::Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        panic!(
            "Incorrect number of arguments!\n{}",
            user_messages::USAGE_ERROR
        );
    }

    let target = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        panic!("Input file does not exist!\n{}", user_messages::USAGE_ERROR);
    }

    match target.as_str() {
        "--assemble" | "-a" => {
            match assembler::start_assembler(input_file, output_file) {
                Ok(_) => println!("File assembled successfully!"),
                Err(err) => {
                    for error in err.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                },
            };
        }
        // "--disassemble" | "-d" => disassembler::disassemble(input_file, output_file),
        _ => panic!(
            "Invalid target \"{}\"!\n{}",
            target,
            user_messages::USAGE_ERROR
        ),
    }

    // println!("Time elapsed: {}ns", now.elapsed().as_nanos());
}
