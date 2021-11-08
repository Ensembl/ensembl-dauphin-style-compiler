use crate::request::core::request::{RequestVariant};
use serde_cbor::Value as CborValue;

pub(crate) struct BootstrapReq;

impl BootstrapReq {
    pub(crate) fn new() -> RequestVariant {
        RequestVariant::Bootstrap(BootstrapReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
