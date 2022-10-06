use serde_cbor::Value as CborValue;

use crate::{request::core::request::{MiniRequest, MiniRequestVariety}};

pub struct BootChannelReq;

impl BootChannelReq {
    pub(crate) fn new() -> MiniRequest {
        MiniRequest::BootChannel(BootChannelReq)
    }

    pub fn encode(&self) -> CborValue { CborValue::Null }
}

impl MiniRequestVariety for BootChannelReq {
    fn description(&self) -> String { "boot".to_string() }
}
