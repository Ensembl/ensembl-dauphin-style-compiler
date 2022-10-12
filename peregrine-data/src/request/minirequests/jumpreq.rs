use crate::request::core::request::{MiniRequest, MiniRequestVariety};
use serde::{Serialize, ser::SerializeSeq};

pub struct JumpReq {
    location: String
}

impl JumpReq {
    pub(crate) fn new(location: &str) -> MiniRequest {
        MiniRequest::Jump(JumpReq {
            location: location.to_string()
        })
    }

    pub fn location(&self) -> &str { &self.location }
}

impl MiniRequestVariety for JumpReq {
    fn description(&self) -> String { "jump".to_string() }
    fn opcode(&self) -> u8 { 5 }
}

impl Serialize for JumpReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.location)?;
        seq.end()
    }
}