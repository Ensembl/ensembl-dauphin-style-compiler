use std::fmt::{ Debug, Display };

pub(crate) enum EarpAssemblerError {
    OpcodeInUse(String),
    DuplicateLabel(String),
    BadOpcodeMap(String),
    UnknownOpcode(String),
    UnknownLabel(String),
    EncodingError(String)
}

impl Debug for EarpAssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpcodeInUse(arg0) => f.debug_tuple("OpcodeInUse").field(arg0).finish(),
            Self::DuplicateLabel(arg0) => f.debug_tuple("DuplicateLabel").field(arg0).finish(),
            Self::UnknownOpcode(arg0) => f.debug_tuple("UnknownOpcode").field(arg0).finish(),
            Self::UnknownLabel(arg0) => f.debug_tuple("UnknownLabel").field(arg0).finish(),
            Self::EncodingError(arg0) => f.debug_tuple("EncodingError").field(arg0).finish(),
            Self::BadOpcodeMap(arg0) => {
                write!(f,"{}",arg0)?;
                Ok(())
            }
        }
    }
}

impl Display for EarpAssemblerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{:?}",self)
    }
}
