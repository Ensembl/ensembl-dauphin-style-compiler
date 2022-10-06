use crate::request::core::request::{MiniRequest, MiniRequestVariety};
use serde_cbor::Value as CborValue;

pub struct JumpReq {
    location: String
}

impl JumpReq {
    pub(crate) fn new(location: &str) -> MiniRequest {
        MiniRequest::Jump(JumpReq {
            location: location.to_string()
        })
    }

    pub fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.location.to_string())
        ])
    }
}

impl MiniRequestVariety for JumpReq {
    fn description(&self) -> String { "jump".to_string() }
}
