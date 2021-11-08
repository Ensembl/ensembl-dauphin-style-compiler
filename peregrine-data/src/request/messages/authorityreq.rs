use crate::request::core::request::{RequestVariant};
use serde_cbor::Value as CborValue;

pub(crate) struct AuthorityCommandRequest;

impl AuthorityCommandRequest {
    pub(crate) fn new() -> RequestVariant {
        RequestVariant::Authority(AuthorityCommandRequest)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
