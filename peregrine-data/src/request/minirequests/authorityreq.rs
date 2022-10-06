use serde::Serialize;

use crate::request::core::request::{MiniRequest, MiniRequestVariety};

pub struct AuthorityReq;

impl AuthorityReq {
    pub(crate) fn new() -> MiniRequest {
        MiniRequest::Authority(AuthorityReq)
    }
}

impl MiniRequestVariety for AuthorityReq {
    fn description(&self) -> String { "authority".to_string() }
    fn opcode(&self) -> u8 { 3 }
}

impl Serialize for AuthorityReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.serialize_none()
    }
}