use peregrine_toolkit::cbor::cbor_as_str;
use serde_cbor::Value as CborValue;

pub struct FailureRes {
    message: String
}

impl FailureRes {
    pub fn new(msg: &str) -> FailureRes {
        FailureRes { message: msg.to_string() }
    }

    pub fn message(&self) -> &str { &self.message }

    pub fn decode(value: CborValue) -> Result<FailureRes,String> {
        Ok(FailureRes { message: cbor_as_str(&value)?.to_string() })
    }
}
