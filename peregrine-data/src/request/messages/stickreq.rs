use serde_cbor::Value as CborValue;
use crate::{StickId, request::core::request::{RequestVariant}};

pub(crate) struct StickReq {
    stick_id: StickId
}

impl StickReq {
    pub(crate) fn new(stick_id: &StickId) -> RequestVariant {
        RequestVariant::Stick(StickReq {
            stick_id: stick_id.clone()
        })
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.stick_id.get_id().to_string())
        ])
    }
}