use std::fmt::{ self, Display, Formatter };
use peregrine_toolkit::{cbor::cbor_as_vec};
use serde_cbor::Value as CborValue;

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct BackendNamespace(String,String);

impl BackendNamespace {
    pub fn new(scheme: &str, rest: &str) -> BackendNamespace {
        BackendNamespace(scheme.to_string(),rest.to_string())
    }

    pub fn or_missing(channel: &Option<BackendNamespace>) -> BackendNamespace {
        channel.clone().unwrap_or_else(|| BackendNamespace::new("",""))
    }

    pub fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Text(self.0.to_string()),
            CborValue::Text(self.1.to_string())
        ])
    }

    pub fn decode(value: CborValue) -> Result<BackendNamespace,String> {
        let parts = cbor_as_vec(&value)?;
        if parts.len() != 2 {
            return Err(format!("bad channel spec: should be two-part array"));
        }
        if let (CborValue::Text(a),CborValue::Text(b)) = (&parts[0],&parts[1]) {
            Ok(BackendNamespace(a.to_string(),b.to_string()))
        } else {
            Err(format!("bad channel name {:?}",value))
        }
    }
}

impl Display for BackendNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}:{}",self.0,self.1)
    }
}
