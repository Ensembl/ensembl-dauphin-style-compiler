use serde::Serializer;
use crate::request::core::request::NewRequestVariant;

pub(crate) struct AuthorityCommandRequest;

impl AuthorityCommandRequest {
    pub(crate) fn new() -> NewRequestVariant {
        NewRequestVariant::Authority(AuthorityCommandRequest)
    }
}

impl serde::Serialize for AuthorityCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}
