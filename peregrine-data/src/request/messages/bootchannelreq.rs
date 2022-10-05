use serde_cbor::Value as CborValue;

use crate::{request::core::request::BackendRequest};

pub(crate) struct BootChannelReq;

impl BootChannelReq {
    pub(crate) fn new() -> BackendRequest {
        BackendRequest::BootChannel(BootChannelReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
