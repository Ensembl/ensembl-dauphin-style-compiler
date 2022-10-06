use serde_cbor::Value as CborValue;

use crate::request::core::request::{MiniRequest, MiniRequestVariety};

pub struct AuthorityReq;

impl AuthorityReq {
    pub(crate) fn new() -> MiniRequest {
        MiniRequest::Authority(AuthorityReq)
    }

    pub fn encode(&self) -> CborValue { CborValue::Null }
}

impl MiniRequestVariety for AuthorityReq {
    fn description(&self) -> String { "authority".to_string() }
}
