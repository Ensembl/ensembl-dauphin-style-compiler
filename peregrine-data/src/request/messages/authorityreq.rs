use crate::request::core::request::{RequestVariant};
use serde_cbor::Value as CborValue;

pub(crate) struct AuthorityReq;

impl AuthorityReq {
    pub(crate) fn new() -> RequestVariant {
        RequestVariant::Authority(AuthorityReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
