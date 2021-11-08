use serde_cbor::Value as CborValue;

pub struct ProgramCommandResponse {}

impl ProgramCommandResponse {
    pub fn decode(_value: CborValue) -> Result<ProgramCommandResponse,String> {
        Ok(ProgramCommandResponse{})
    }
}
