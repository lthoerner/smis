pub enum AssemblerError {
    FileHandlerError(FileHandlerError),
    ParseError(ParseError)
}

impl From<FileHandlerError> for AssemblerError {
    fn from(error: FileHandlerError) -> Self {
        AssemblerError::FileHandlerError(error)
    }
}

impl From<ParseError> for AssemblerError {
    fn from(error: ParseError) -> Self {
        AssemblerError::ParseError(error)
    }
}

pub enum FileHandlerError {
    ErrorFileOpenFailed,
    ErrorFileCreateFailed,
    ErrorInvalidFileContents
}

pub enum ParseError {
    MnemonicParseError(MnemonicParseError),
    RegisterParseError(RegisterParseError),
    ImmediateParseError(ImmediateParseError)
}

impl From<MnemonicParseError> for ParseError {
    fn from(error: MnemonicParseError) -> Self {
        ParseError::MnemonicParseError(error)
    }
}

impl From<RegisterParseError> for ParseError {
    fn from(error: RegisterParseError) -> Self {
        ParseError::RegisterParseError(error)
    }
}

impl From<ImmediateParseError> for ParseError {
    fn from(error: ImmediateParseError) -> Self {
        ParseError::ImmediateParseError(error)
    }
}

pub enum MnemonicParseError {
    ErrorMnemonicNotFound
}

pub enum RegisterParseError {
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    ErrorNumberOutOfBounds
}

pub enum ImmediateParseError {
    ErrorInvalidPrefix,
    ErrorNonNumeric,
    ErrorNumberOutOfBounds
}