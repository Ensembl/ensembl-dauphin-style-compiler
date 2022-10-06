use serde_cbor::Value as CborValue;
use crate::{StickId, request::core::request::{MiniRequest, MiniRequestVariety}};

pub struct StickReq {
    stick_id: StickId
}

impl StickReq {
    pub(crate) fn new(stick_id: &StickId) -> MiniRequest {
        MiniRequest::Stick(StickReq {
            stick_id: stick_id.clone()
        })
    }

    pub fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.stick_id.get_id().to_string())
        ])
    }
}

impl MiniRequestVariety for StickReq {
    fn description(&self) -> String { "boot".to_string() }
}
