use serde_cbor::Value as CborValue;
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,Debug,Eq,PartialEq,Hash)]
pub struct Scale(u64);

impl Scale {
    pub fn new(scale: u64) -> Scale {
        Scale(scale)
    }

    pub fn new_for_numeric(scale: f64) -> Scale {
        Scale::new(scale.round() as u64)
    }

    pub fn prev_scale(&self) -> Scale {
        Scale(self.0-1)
    }

    pub fn next_scale(&self) -> Scale {
        Scale(self.0+1)
    }

    pub fn get_index(&self) -> u64 {
        self.0
    }

    pub fn bp_in_carriage(&self) -> u64 {
        1 << self.0
    }

    pub fn carriage(&self, position: f64) -> u64 {
        (position / (self.bp_in_carriage() as f64)).floor() as u64
    }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Integer(self.0 as i128))
    }
}

impl Display for Scale {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}
