use minicbor::{Encoder};

use crate::{parser::EarpAssemblyOperand, error::EarpAssemblerError, assemble::Assemble};

pub(crate) enum EarpOperand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl EarpOperand {
    pub(crate) fn new(value: &EarpAssemblyOperand, context: &Assemble) -> Result<EarpOperand,EarpAssemblerError> {
        Ok(match value {
            EarpAssemblyOperand::Register(r) => EarpOperand::Register(*r),
            EarpAssemblyOperand::UpRegister(r) => EarpOperand::UpRegister(*r),
            EarpAssemblyOperand::String(s) => EarpOperand::String(s.clone()),
            EarpAssemblyOperand::Boolean(b) => EarpOperand::Boolean(*b),
            EarpAssemblyOperand::Integer(n) => EarpOperand::Integer(*n),
            EarpAssemblyOperand::Float(f) => EarpOperand::Float(*f),
            EarpAssemblyOperand::Location(loc) => EarpOperand::Integer(context.resolve_label(loc)?)
        })
    }

    /* up to 23 gets single byte, ie *; rx,*; ux,* */
    fn type_value(&self) -> u64 {
        match self {
            EarpOperand::Register(_) => { 1 },
            EarpOperand::UpRegister(_) => { 2 },
            EarpOperand::String(_) => { 3 },
            EarpOperand::Boolean(_) => { 4 },
            EarpOperand::Integer(_) => { 5 },
            EarpOperand::Float(_) => { 6 },
        }
    }

    fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),minicbor::encode::Error<std::io::Error>> {
        match self {
            EarpOperand::Register(v) => { encoder.i64(*v as i64)?; },
            EarpOperand::UpRegister(v) => { encoder.i64(*v as i64)?; },
            EarpOperand::String(s) => { encoder.str(s)?; },
            EarpOperand::Boolean(b) => { encoder.bool(*b)?; },
            EarpOperand::Integer(v) => { encoder.i64(*v)?; },
            EarpOperand::Float(v) => { encoder.f64(*v)?; },
        }
        Ok(())
    }
}

pub(crate) struct EarpCommand(pub u64,pub Vec<EarpOperand>);

impl EarpCommand {
    fn type_value(&self) -> u64 {
        let mut out = 0;
        for arg in &self.1 {
            out = out*8 + arg.type_value();
        }
        out
    }

    pub(crate) fn encode(&self, encoder: &mut Encoder<&mut Vec<u8>>) -> Result<(),minicbor::encode::Error<std::io::Error>> {
        encoder.u64(self.0)?;
        encoder.u64(self.type_value())?;
        for arg in &self.1 {
            arg.encode(encoder)?;
        }
        Ok(())
    }
}
