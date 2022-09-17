use serde_cbor::Value as CborValue;

use crate::request::core::request::BackendRequest;

pub(crate) struct AuthorityReq;

impl AuthorityReq {
    pub(crate) fn new() -> BackendRequest {
        BackendRequest::Authority(AuthorityReq)
    }

    pub(crate) fn encode(&self) -> CborValue { CborValue::Null }
}
