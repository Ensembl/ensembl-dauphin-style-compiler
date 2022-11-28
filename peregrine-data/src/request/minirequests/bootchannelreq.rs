use serde::Serialize;

use crate::{request::core::minirequest::{MiniRequest, MiniRequestVariety}};

pub struct BootChannelReq;

impl BootChannelReq {
    pub(crate) fn new() -> MiniRequest {
        MiniRequest::BootChannel(BootChannelReq)
    }
}

impl MiniRequestVariety for BootChannelReq {
    fn description(&self) -> String { "boot".to_string() }
    fn opcode(&self) -> u8 { 0 }
}

impl Serialize for BootChannelReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.serialize_none()
    }
}