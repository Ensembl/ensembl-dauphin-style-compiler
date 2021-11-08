use crate::Stick;
use serde_cbor::Value as CborValue;

pub struct StickRes {
    stick: Stick
}

impl StickRes {
    pub(crate) fn stick(&self) -> Stick { self.stick.clone() }

    pub(crate) fn decode(value: CborValue) -> Result<StickRes,String> {
        Ok(StickRes {
            stick: Stick::decode(value)?
        })
    }
}
