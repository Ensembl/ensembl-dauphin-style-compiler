use peregrine_toolkit::envaryseq;
use serde::Serializer;

use crate::request::core::request::NewRequestVariant;

pub struct JumpCommandRequest {
    location: String
}

impl JumpCommandRequest {
    pub(crate) fn new(location: &str) -> NewRequestVariant {
        NewRequestVariant::Jump(JumpCommandRequest {
            location: location.to_string()
        })
    }
}

impl serde::Serialize for JumpCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        envaryseq!(serializer,self.location.to_string())
    }
}
