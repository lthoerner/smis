// This is here because the compiler complains about the crate being called "SMIS" instead of "smis",
// despite the fact that it's an acronym and should be all caps
#![allow(non_snake_case)]

use std::path::Path;
use std::process::exit;
use utilities::messages;

mod assembler;
mod disassembler;
mod emulator;
mod utilities;

fn main() {
    // let now = std::time::Instant::now();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        println!("Incorrect number of arguments!\n{}", messages::USAGE);
        exit(1);
    }

    let target = &args[1];
    let input_file = &args[2];
    let output_file = &args[3];

    if !Path::new(input_file).exists() {
        println!("Input file does not exist!\n{}", messages::USAGE);
        exit(2);
    }

    match target.as_str() {
        "assemble" => {
            match assembler::start_assembler(input_file, output_file) {
                Ok(_) => println!("File assembled successfully!"),
                Err(e) => {
                    for error in e.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                }
            };
        }
        "disassemble" => {
            match disassembler::start_disassembler(input_file, output_file) {
                Ok(_) => println!("File disassembled successfully!"),
                Err(e) => {
                    for error in e.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                }
            };
        }
        _ => {
            println!("Invalid target \"{}\"!\n{}", target, messages::USAGE);
            exit(3);
        }
    }

    // println!("Time elapsed: {}ns", now.elapsed().as_nanos());
}
