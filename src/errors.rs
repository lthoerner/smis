pub enum FileHandlerError {
    ErrorFileOpenFailed,
    ErrorFileCreateFailed,
    ErrorFileWriteFailed,
}

pub enum MnemonicParseError {
    ErrorUnknownMnemonic,
}

pub enum RegisterParseError {
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    ErrorInvalidNumber,
}

pub enum ImmediateParseError {
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    ErrorInvalidNumber,
}