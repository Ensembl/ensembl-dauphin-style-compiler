use std::fmt;
use peregrine_toolkit::{cbor::{cbor_as_str, cbor_into_vec, check_array_len}, decompose_vec};
use serde_cbor::Value as CborValue;

use crate::{BackendNamespace};

#[derive(Clone,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct ProgramName(pub BackendNamespace,pub String);

impl ProgramName {
    pub fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            self.0.encode(),
            CborValue::Text(self.1.clone())
        ])
    }

    pub fn decode(value: CborValue) -> Result<ProgramName,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,2)?;
        decompose_vec!(seq,channel,name);
        Ok(ProgramName(BackendNamespace::decode(channel)?,cbor_as_str(&name)?.to_string()))
    }
}

impl fmt::Display for ProgramName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}/{}",self.0,self.1)
    }
}
