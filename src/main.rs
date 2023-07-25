// This is here because the compiler complains about the crate being called "SMIS" instead of "smis",
// despite the fact that it's an acronym and should be all caps
#![allow(non_snake_case)]

use args::{AssembleCommand, DisassembleCommand, RunCommand, SmisArgs, SmisSubcommand};
use clap::Parser;
use std::path::Path;
use std::process::exit;

mod args;
mod assembler;
mod disassembler;
mod emulator;
mod utilities;

fn main() {
    let start_time = std::time::Instant::now();

    // TODO: Deduplicate error handling
    let args = SmisArgs::parse();
    match args.subcommand {
        SmisSubcommand::Assemble(AssembleCommand {
            input_filename,
            output_filename,
        }) => {
            assert_file_exists(&input_filename);

            match assembler::start_assembler(&input_filename, &output_filename) {
                Ok(_) => println!(
                    "File assembled successfully in {}ns",
                    start_time.elapsed().as_nanos()
                ),
                Err(e) => {
                    for error in e.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                }
            };
        }
        SmisSubcommand::Disassemble(DisassembleCommand {
            input_filename,
            output_filename,
        }) => {
            assert_file_exists(&input_filename);

            match disassembler::start_disassembler(&input_filename, &output_filename) {
                Ok(_) => println!(
                    "File disassembled successfully in {}ns",
                    start_time.elapsed().as_nanos()
                ),
                Err(e) => {
                    for error in e.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                }
            };
        }
        SmisSubcommand::Run(RunCommand {
            machine_code_filename,
        }) => {
            assert_file_exists(&machine_code_filename);

            match emulator::start_emulator(&machine_code_filename) {
                Ok(_) => println!(
                    "Program run successfully in {}ns",
                    start_time.elapsed().as_nanos()
                ),
                Err(e) => {
                    for error in e.chain().rev().skip(1) {
                        println!("{}", error);
                    }
                }
            };
        }
    }
}

fn assert_file_exists(filename: &str) {
    if !Path::new(filename).exists() {
        println!("File '{}' does not exist!", filename);
        exit(2);
    }
}
