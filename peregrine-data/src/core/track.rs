use serde_cbor::Value as CborValue;
use std::cmp::Ord;
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,Debug,Hash,PartialEq,Eq,PartialOrd,Ord)]
pub struct Track(String);

impl Track {
    pub fn new(name: &str) -> Track {
        Track(name.to_string())
    }

    pub fn name(&self) -> &str { &self.0 }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Text(self.0.clone()))
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}