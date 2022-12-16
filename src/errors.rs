#[derive(Debug, thiserror::Error)]
#[error("Encountered an error while handling a file.")]
pub enum FileHandlerError {
    ErrorInvalidExtension,
    ErrorFileOpenFailed,
    ErrorFileCreateFailed,
    ErrorFileReadFailed,
    ErrorFileWriteFailed,
    ErrorFileRewindFailed,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing a mnemonic.")]
pub enum MnemonicParseError {
    ErrorInvalidIndex,
    ErrorUnknownMnemonic,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing an opcode.")]
pub enum OpcodeParseError {
    ErrorUnknownOpcode,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing a register.")]
pub enum RegisterParseError {
    ErrorInvalidIndex,
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    ErrorInvalidNumber,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when parsing an immediate.")]
pub enum ImmediateParseError {
    ErrorInvalidIndex,
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    // ErrorInvalidNumber,
}

#[derive(Debug, thiserror::Error)]
#[error("Encountered an error when operating on the symbol table.")]
pub enum SymbolTableError {
    ErrorCouldNotAddLabel,
    ErrorLabelNotFound,
    // TODO: Add behavior for this
    // ErrorLabelAlreadyExists,
}
