use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct SmisArgs {
    #[clap(subcommand)]
    /// The type of operation to perform
    pub subcommand: SmisSubcommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum SmisSubcommand {
    Assemble(AssembleCommand),
    Disassemble(DisassembleCommand),
    Run(RunCommand),
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Assemble a .txt assembly code file into a .bin machine code file")]
pub struct AssembleCommand {
    /// The input file to assemble
    pub input_filename: String,
    /// The output file to write the assembled machine code to
    pub output_filename: String,
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Disassemble a .bin machine code file into a .txt assembly code file")]
pub struct DisassembleCommand {
    /// The input file to disassemble
    pub input_filename: String,
    /// The output file to write the disassembled assembly code to
    pub output_filename: String,
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Run a .bin machine code file")]
pub struct RunCommand {
    /// The machine code file to run
    pub machine_code_filename: String,
}
