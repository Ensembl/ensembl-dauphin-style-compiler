use crate::{Stick, core::stick::StickResponse};
use serde_cbor::Value as CborValue;

pub struct StickRes {
    stick: Result<Stick,String>
}

impl StickRes {
    pub(crate) fn stick(&self) -> Result<Stick,String> { self.stick.clone() }

    pub(crate) fn decode(value: CborValue) -> Result<StickRes,String> {
        let response = match StickResponse::decode(value)? {
            StickResponse::Stick(s) => Ok(s),
            StickResponse::Unknown(u) => Err(u)
        };
        Ok(StickRes {
            stick: response
        })
    }
}
