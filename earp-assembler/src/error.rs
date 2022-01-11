use std::fmt::{ Debug, Display };

#[derive(Clone)]
pub(crate) enum EarpAssemblerError {
    OpcodeInUse(String),
    DuplicateLabel(String),
    BadOpcodeMap(String),
    UnknownOpcode(String),
    UnknownLabel(String),
    EncodingError(String),
    BadHexFile(String),
    FileError(String),
    SyntaxError(String),
    BadHereLabel(String)
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
    BadHereLabel
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
            AssemblerErrorType::BadHexFile => "BadHex File",
            AssemblerErrorType::FileError => "File Error",
            AssemblerErrorType::SyntaxError => "Syntax Error",
            AssemblerErrorType::BadHereLabel => "Bad Here Label"
        }
    }

    fn unburst(&self, msg: String) -> EarpAssemblerError {
        match self {
            AssemblerErrorType::OpcodeInUse => EarpAssemblerError::OpcodeInUse(msg),
            AssemblerErrorType::DuplicateLabel => EarpAssemblerError::DuplicateLabel(msg),
            AssemblerErrorType::BadOpcodeMap => EarpAssemblerError::BadOpcodeMap(msg),
            AssemblerErrorType::UnknownOpcode => EarpAssemblerError::UnknownOpcode(msg),
            AssemblerErrorType::UnknownLabel => EarpAssemblerError::UnknownLabel(msg),
            AssemblerErrorType::EncodingError => EarpAssemblerError::EncodingError(msg),
            AssemblerErrorType::BadHexFile => EarpAssemblerError::BadHexFile(msg),
            AssemblerErrorType::FileError => EarpAssemblerError::FileError(msg),
            AssemblerErrorType::SyntaxError => EarpAssemblerError::SyntaxError(msg),
            AssemblerErrorType::BadHereLabel => EarpAssemblerError::BadHereLabel(msg),
        }
    }
}

struct Burst(AssemblerErrorType,String);

impl EarpAssemblerError {    
    fn burst(self) -> Burst {
        match self {
            EarpAssemblerError::OpcodeInUse(s) => Burst(AssemblerErrorType::OpcodeInUse,s),
            EarpAssemblerError::DuplicateLabel(s) => Burst(AssemblerErrorType::DuplicateLabel,s),
            EarpAssemblerError::BadOpcodeMap(s) => Burst(AssemblerErrorType::BadOpcodeMap,s),
            EarpAssemblerError::UnknownOpcode(s) => Burst(AssemblerErrorType::UnknownOpcode,s),
            EarpAssemblerError::UnknownLabel(s) => Burst(AssemblerErrorType::UnknownLabel,s),
            EarpAssemblerError::EncodingError(s) => Burst(AssemblerErrorType::EncodingError,s),
            EarpAssemblerError::BadHexFile(s) => Burst(AssemblerErrorType::BadHexFile,s),
            EarpAssemblerError::FileError(s) => Burst(AssemblerErrorType::FileError,s),
            EarpAssemblerError::SyntaxError(s) => Burst(AssemblerErrorType::SyntaxError,s),
            EarpAssemblerError::BadHereLabel(s) => Burst(AssemblerErrorType::BadHereLabel,s),
        }
    }

    pub(crate) fn add_context(&self, context: &str) -> EarpAssemblerError {
        let burst = self.clone().burst();
        burst.0.unburst(format!("{}:\n{}",context,burst.1))
    }
}

impl Debug for EarpAssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let burst = self.clone().burst();
        write!(f,"{}: {}",burst.0.kind(),burst.1)
    }
}

impl Display for EarpAssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self)
    }
}

pub(crate) fn opcode_error(prefix: Option<String>, name: &str) -> EarpAssemblerError {
    EarpAssemblerError::UnknownOpcode(
        if let Some(prefix) = prefix {
            format!("{}:{}",prefix,name)
        } else {
            name.to_string()
        }
    )
}
