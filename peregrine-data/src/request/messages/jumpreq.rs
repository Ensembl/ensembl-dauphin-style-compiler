use crate::request::core::request::{RequestVariant};
use serde_cbor::Value as CborValue;

pub struct JumpCommandRequest {
    location: String
}

impl JumpCommandRequest {
    pub(crate) fn new(location: &str) -> RequestVariant {
        RequestVariant::Jump(JumpCommandRequest {
            location: location.to_string()
        })
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.location.to_string())
        ])
    }
}
