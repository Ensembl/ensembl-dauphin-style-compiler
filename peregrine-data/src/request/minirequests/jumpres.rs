use peregrine_toolkit::cbor::{cbor_as_number, cbor_as_str, cbor_into_map, cbor_map_contains, cbor_map_key};
use serde_cbor::Value as CborValue;

pub struct JumpLocation {
    pub stick: String,
    pub left: u64,
    pub right: u64
}

pub enum JumpRes {
    Found(JumpLocation),
    NotFound
}

impl JumpRes {
    pub fn decode(value: CborValue) -> Result<JumpRes,String> {
        let mut map = cbor_into_map(value)?;
        if cbor_map_contains(&map, "no") {
            Ok(JumpRes::NotFound)
        } else {
            Ok(JumpRes::Found(JumpLocation {
                stick: cbor_as_str(&cbor_map_key(&mut map,"stick")?)?.to_string(),
                left: cbor_as_number(&cbor_map_key(&mut map,"left")?)?,
                right: cbor_as_number(&cbor_map_key(&mut map,"right")?)?,
            }))
        }
    }
}
