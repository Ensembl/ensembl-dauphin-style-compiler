use crate::{request::core::request::{MiniRequest, MiniRequestVariety}};
use serde::{Serialize, ser::SerializeSeq};

pub struct ExpandReq {
    name: String
}

impl ExpandReq {
    pub(crate) fn new(name: &str) -> MiniRequest {
        MiniRequest::Expand(ExpandReq {
            name: name.to_string()
        })
    }    
}

impl MiniRequestVariety for ExpandReq {
    fn description(&self) -> String { "expand".to_string() }
    fn opcode(&self) -> u8 { 7 }
}

impl Serialize for ExpandReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.name)?;
        seq.end()
    }
}