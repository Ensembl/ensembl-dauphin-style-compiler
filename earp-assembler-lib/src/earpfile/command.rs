use minicbor::{Encoder, Encode};

use crate::{core::error::AssemblerError, assemble::{parser::ParseOperand, assembler::AssembleFile}, Assemble};

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum Operand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Operand {
    pub(crate) fn new(value: &ParseOperand, assemble: &Assemble, context: &AssembleFile) -> Result<Operand,AssemblerError> {
        Ok(match value {
            ParseOperand::Register(r) => Operand::Register(*r),
            ParseOperand::UpRegister(r) => Operand::UpRegister(*r),
            ParseOperand::String(s) => Operand::String(s.clone()),
            ParseOperand::Boolean(b) => Operand::Boolean(*b),
            ParseOperand::Integer(n) => Operand::Integer(*n),
            ParseOperand::Float(f) => Operand::Float(*f),
            ParseOperand::Location(loc) => Operand::Integer(context.resolve_label(assemble,loc)?)
        })
    }

    fn type_value(&self) -> u64 {
        match self {
            Operand::Register(_) => { 1 },
            Operand::UpRegister(_) => { 2 },
            Operand::String(_) => { 3 },
            Operand::Boolean(_) => { 3 },
            Operand::Integer(_) => { 3 },
            Operand::Float(_) => { 3 },
        }
    }
}

impl Encode for Operand {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        match self {
            Operand::Register(v) => { encoder.i64(*v as i64)?; },
            Operand::UpRegister(v) => { encoder.i64(*v as i64)?; },
            Operand::String(s) => { encoder.str(s)?; },
            Operand::Boolean(b) => { encoder.bool(*b)?; },
            Operand::Integer(v) => { encoder.i64(*v)?; },
            Operand::Float(v) => { encoder.f64(*v)?; },
        }
        Ok(())
    }
}

#[derive(Clone,Debug,PartialEq)]
pub(crate) struct Command(pub u64,pub Vec<Operand>);

impl Command {
    pub(crate) fn type_value(&self) -> u64 {
        let mut out = 0;
        for arg in self.1.iter().rev() {
            out = out*4 + arg.type_value();
        }
        out
    }
}

impl Encode for Command {
    fn encode<W: minicbor::encode::Write>(&self, encoder: &mut Encoder<W>) -> Result<(), minicbor::encode::Error<W::Error>> {
        encoder.u64(self.0)?.u64(self.type_value())?;
        for arg in &self.1 {
            encoder.encode(arg)?;
        }
        Ok(())
    }
}
