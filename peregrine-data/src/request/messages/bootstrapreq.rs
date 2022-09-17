use serde_cbor::Value as CborValue;

use crate::request::core::request::BackendRequest;

pub(crate) struct BootstrapReq;

impl BootstrapReq {
    pub(crate) fn new() -> BackendRequest {
        BackendRequest::Bootstrap(BootstrapReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
