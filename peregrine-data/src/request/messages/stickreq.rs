use peregrine_toolkit::envaryseq;
use serde::Serializer;

use crate::{StickId, request::core::request::NewRequestVariant};

pub(crate) struct StickCommandRequest {
    stick_id: StickId
}

impl StickCommandRequest {
    pub(crate) fn new(stick_id: &StickId) -> NewRequestVariant {
        NewRequestVariant::Stick(StickCommandRequest {
            stick_id: stick_id.clone()
        })
    }
}

impl serde::Serialize for StickCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        envaryseq!(serializer,self.stick_id.get_id().to_string())
    }
}
