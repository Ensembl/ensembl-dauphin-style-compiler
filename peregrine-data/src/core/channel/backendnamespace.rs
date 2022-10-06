use std::fmt::{ self, Display, Formatter };
use peregrine_toolkit::{cbor::cbor_as_vec};
use serde::{Serialize, ser::SerializeSeq};
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

impl Serialize for BackendNamespace {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(2))?;
        seq.serialize_element(&self.0)?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}

impl Display for BackendNamespace {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f,"{}:{}",self.0,self.1)
    }
}
