use peregrine_toolkit::cbor::cbor_as_str;
use serde_cbor::Value as CborValue;

pub struct GeneralFailure {
    message: String
}

impl GeneralFailure {
    pub fn new(msg: &str) -> GeneralFailure {
        GeneralFailure { message: msg.to_string() }
    }

    pub fn message(&self) -> &str { &self.message }

    pub fn decode(value: CborValue) -> Result<GeneralFailure,String> {
        Ok(GeneralFailure { message: cbor_as_str(&value)?.to_string() })
    }
}
