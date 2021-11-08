use crate::Stick;
use serde_cbor::Value as CborValue;

pub struct StickCommandResponse {
    stick: Stick
}

impl StickCommandResponse {
    pub(crate) fn stick(&self) -> Stick { self.stick.clone() }

    pub(crate) fn decode(value: CborValue) -> Result<StickCommandResponse,String> {
        Ok(StickCommandResponse {
            stick: Stick::decode(value)?
        })
    }
}
