use minicbor::{Decoder, decode::Error};

#[derive(Clone,Debug,PartialEq)]
pub enum Operand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Operand {
    pub(crate) fn decode(variety: usize, decoder: &mut Decoder) -> Result<Operand,minicbor::decode::Error> {
        Ok(if decoder.probe().i64().is_ok() {
            let value = decoder.i64()?;
            match variety {
                1 => Operand::Register(value as usize),
                2 => Operand::UpRegister(value as usize),
                _ => Operand::Integer(value)
            }
        } else if decoder.probe().str().is_ok() {
            Operand::String(decoder.str()?.to_string())
        } else if decoder.probe().bool().is_ok() {
            Operand::Boolean(decoder.bool()?)
        } else if decoder.probe().f64().is_ok() {
            Operand::Float(decoder.f64()?)
        } else {
            return Err(Error::Message("unexpected operand"));
        })
    }
}
