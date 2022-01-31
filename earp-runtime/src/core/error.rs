#[cfg_attr(debug_assertions,derive(Debug))]
pub enum EarpError {
    BadEarpFile(String),
    DuplicateInstruction(String),
    BadOpcode(String),
    BadMagic(String),
}

pub struct EarpFault(pub String);
