use crate::request::core::request::{RequestVariant};
use serde_cbor::Value as CborValue;

pub(crate) struct BootstrapCommandRequest;

impl BootstrapCommandRequest {
    pub(crate) fn new() -> RequestVariant {
        RequestVariant::Bootstrap(BootstrapCommandRequest)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
