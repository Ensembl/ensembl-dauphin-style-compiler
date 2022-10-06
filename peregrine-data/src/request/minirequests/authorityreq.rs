use serde_cbor::Value as CborValue;

use crate::request::core::request::{MiniRequest, MiniRequestVariety};

pub(crate) struct AuthorityReq;

impl AuthorityReq {
    pub(crate) fn new() -> MiniRequest {
        MiniRequest::Authority(AuthorityReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}

impl MiniRequestVariety for AuthorityReq {
    fn description(&self) -> String { "authority".to_string() }
}
