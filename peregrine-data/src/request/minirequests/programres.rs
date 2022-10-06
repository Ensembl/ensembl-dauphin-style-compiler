use serde_cbor::Value as CborValue;

pub struct ProgramRes {}

impl ProgramRes {
    pub fn decode(_value: CborValue) -> Result<ProgramRes,String> {
        Ok(ProgramRes{})
    }
}
