use serde::Serializer;
use crate::request::core::request::NewRequestVariant;

pub(crate) struct BootstrapCommandRequest;

impl BootstrapCommandRequest {
    pub(crate) fn new() -> NewRequestVariant {
        NewRequestVariant::Bootstrap(BootstrapCommandRequest)
    }
}

impl serde::Serialize for BootstrapCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}
