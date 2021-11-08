use serde_cbor::Value as CborValue;
use crate::{StickId, request::core::request::{RequestVariant}};

pub(crate) struct StickCommandRequest {
    stick_id: StickId
}

impl StickCommandRequest {
    pub(crate) fn new(stick_id: &StickId) -> RequestVariant {
        RequestVariant::Stick(StickCommandRequest {
            stick_id: stick_id.clone()
        })
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.stick_id.get_id().to_string())
        ])
    }
}
