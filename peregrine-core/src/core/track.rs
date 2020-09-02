use serde_cbor::Value as CborValue;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
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