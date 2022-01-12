use std::fmt::{ Debug, Display };

#[derive(Clone)]
pub(crate) enum AssemblerError {
    OpcodeInUse(String),
    DuplicateLabel(String),
    BadOpcodeMap(String),
    UnknownOpcode(String),
    UnknownLabel(String),
    EncodingError(String),
    BadHexFile(String),
    FileError(String),
    SyntaxError(String),
    BadHereLabel(String),
    CannotSerialize(String),
    DuplicateOpcode(String),
    DuplicateAssetName(String)
}

#[derive(Clone)]
pub(crate) enum AssemblerErrorType {
    OpcodeInUse,
    DuplicateLabel,
    BadOpcodeMap,
    UnknownOpcode,
    UnknownLabel,
    EncodingError,
    BadHexFile,
    FileError,
    SyntaxError,
    BadHereLabel,
    CannotSerialize,
    DuplicateOpcode,
    DuplicateAssetName
}

impl AssemblerErrorType {
    fn kind(&self) -> &str {
        match self {
            AssemblerErrorType::OpcodeInUse => "Opcode Already In Use",
            AssemblerErrorType::DuplicateLabel => "Duplicate Label",
            AssemblerErrorType::BadOpcodeMap => "Bad Opcode Map",
            AssemblerErrorType::UnknownOpcode => "Unknown Opcode",
            AssemblerErrorType::UnknownLabel => "Unknown Label",
            AssemblerErrorType::EncodingError => "Encoding Error",
            AssemblerErrorType::BadHexFile => "Bad Hex File",
            AssemblerErrorType::FileError => "File Error",
            AssemblerErrorType::SyntaxError => "Syntax Error",
            AssemblerErrorType::BadHereLabel => "Bad Here Label",
            AssemblerErrorType::CannotSerialize => "Cannot Serialize",
            AssemblerErrorType::DuplicateOpcode => "Duplicate Opcode",
            AssemblerErrorType::DuplicateAssetName => "Duplicate Asset Name"
        }
    }

    fn unburst(&self, msg: String) -> AssemblerError {
        match self {
            AssemblerErrorType::OpcodeInUse => AssemblerError::OpcodeInUse(msg),
            AssemblerErrorType::DuplicateLabel => AssemblerError::DuplicateLabel(msg),
            AssemblerErrorType::BadOpcodeMap => AssemblerError::BadOpcodeMap(msg),
            AssemblerErrorType::UnknownOpcode => AssemblerError::UnknownOpcode(msg),
            AssemblerErrorType::UnknownLabel => AssemblerError::UnknownLabel(msg),
            AssemblerErrorType::EncodingError => AssemblerError::EncodingError(msg),
            AssemblerErrorType::BadHexFile => AssemblerError::BadHexFile(msg),
            AssemblerErrorType::FileError => AssemblerError::FileError(msg),
            AssemblerErrorType::SyntaxError => AssemblerError::SyntaxError(msg),
            AssemblerErrorType::BadHereLabel => AssemblerError::BadHereLabel(msg),
            AssemblerErrorType::CannotSerialize => AssemblerError::CannotSerialize(msg),
            AssemblerErrorType::DuplicateOpcode => AssemblerError::DuplicateOpcode(msg),
            AssemblerErrorType::DuplicateAssetName => AssemblerError::DuplicateAssetName(msg),
        }
    }
}

struct Burst(AssemblerErrorType,String);

impl AssemblerError {    
    fn burst(self) -> Burst {
        match self {
            AssemblerError::OpcodeInUse(s) => Burst(AssemblerErrorType::OpcodeInUse,s),
            AssemblerError::DuplicateLabel(s) => Burst(AssemblerErrorType::DuplicateLabel,s),
            AssemblerError::BadOpcodeMap(s) => Burst(AssemblerErrorType::BadOpcodeMap,s),
            AssemblerError::UnknownOpcode(s) => Burst(AssemblerErrorType::UnknownOpcode,s),
            AssemblerError::UnknownLabel(s) => Burst(AssemblerErrorType::UnknownLabel,s),
            AssemblerError::EncodingError(s) => Burst(AssemblerErrorType::EncodingError,s),
            AssemblerError::BadHexFile(s) => Burst(AssemblerErrorType::BadHexFile,s),
            AssemblerError::FileError(s) => Burst(AssemblerErrorType::FileError,s),
            AssemblerError::SyntaxError(s) => Burst(AssemblerErrorType::SyntaxError,s),
            AssemblerError::BadHereLabel(s) => Burst(AssemblerErrorType::BadHereLabel,s),
            AssemblerError::CannotSerialize(s) => Burst(AssemblerErrorType::CannotSerialize,s),
            AssemblerError::DuplicateOpcode(s) => Burst(AssemblerErrorType::DuplicateOpcode,s),
            AssemblerError::DuplicateAssetName(s) => Burst(AssemblerErrorType::DuplicateAssetName,s),
        }
    }

    pub(crate) fn add_context(&self, context: &str) -> AssemblerError {
        let burst = self.clone().burst();
        burst.0.unburst(format!("{}: {}",context,burst.1))
    }
}

impl Debug for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let burst = self.clone().burst();
        write!(f,"{}: {}",burst.0.kind(),burst.1)
    }
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self)
    }
}

fn error_fmt(prefix: &Option<String>, name: &str) -> String {
    if let Some(prefix) = prefix {
        format!("{}:{}",prefix,name)
    } else {
        name.to_string()
    }
}

pub(crate) fn unknown_opcode_error(prefix: &Option<String>, name: &str) -> AssemblerError {
    AssemblerError::UnknownOpcode(error_fmt(prefix,name))
}

pub(crate) fn duplicate_opcode_error(prefix: &Option<String>, name: &str) -> AssemblerError {
    AssemblerError::DuplicateOpcode(error_fmt(prefix,name))
}
