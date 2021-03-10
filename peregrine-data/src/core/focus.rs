use serde_cbor::Value as CborValue;
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct Focus(Option<String>);

impl Focus {
    pub fn new(name: Option<&str>) -> Focus {
        Focus(name.map(|x| x.to_string()))
    }

    pub fn name(&self) -> Option<&str> { self.0.as_ref().map(|x| x as &str) }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        if let Some(focus) = &self.0 {
            Ok(CborValue::Text(focus.clone()))
        } else {
            Ok(CborValue::Null)
        }
    }
}

impl Display for Focus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0.as_ref().map(|x| &x as &str).unwrap_or("none"))
    }
}
