#[derive(Debug, thiserror::Error)]
#[error("Encountered an error while handling a file.")]
pub enum FileHandlerError {
    InvalidExtension,
    FileOpenFailed,
    FileCreateFailed,
    FileReadFailed,
    FileWriteFailed,
    FileRewindFailed,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing a mnemonic.")]
pub enum MnemonicParseError {
    InvalidIndex,
    UnknownMnemonic,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing an opcode.")]
pub enum OpcodeParseError {
    UnknownOpcode,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing a register.")]
pub enum RegisterParseError {
    InvalidIndex,
    InvalidPrefix,
    NonNumeric,
    InvalidNumber,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing an immediate.")]
pub enum ImmediateParseError {
    InvalidIndex,
    InvalidPrefix,
    NonNumeric,
    // InvalidNumber,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when operating on the symbol table.")]
pub enum SymbolTableError {
    CouldNotAddLabel,
    LabelNotFound,
    // TODO: Add behavior for this
    // LabelAlreadyExists,
}
