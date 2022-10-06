use serde::{Serialize, ser::SerializeSeq};
use crate::{StickId, request::core::request::{MiniRequest, MiniRequestVariety}};

pub struct StickReq {
    stick_id: StickId
}

impl StickReq {
    pub(crate) fn new(stick_id: &StickId) -> MiniRequest {
        MiniRequest::Stick(StickReq {
            stick_id: stick_id.clone()
        })
    }
}

impl MiniRequestVariety for StickReq {
    fn description(&self) -> String { "boot".to_string() }
    fn opcode(&self) -> u8 { 2 }
}

impl Serialize for StickReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(1))?;
        seq.serialize_element(&self.stick_id)?;
        seq.end()
    }
}
