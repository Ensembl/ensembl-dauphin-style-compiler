use anyhow::bail;
use std::collections::HashSet;
use serde_cbor::Value as CborValue;
use std::fmt::{ self, Display, Formatter };
use std::sync::Arc;
use crate::util::message::DataMessage;
use crate::switch::allotment::{ AllotmentRequest };
#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct StickId(String);

impl StickId {
    pub fn new(name: &str) -> StickId {
        StickId(name.to_string())
    }

    pub fn get_id(&self) -> &str { &self.0 }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Text(self.0.clone()))
    }
}


impl Display for StickId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

#[derive(Clone,Debug)]
pub enum StickTopology {
    Linear,
    Circular
}

impl StickTopology {
    pub fn from_number(n: u8) -> anyhow::Result<StickTopology> {
        Ok(match n {
            0 => StickTopology::Linear,
            1 => StickTopology::Circular,
            _ => bail!("bad topology number")
        })
    }

    pub fn to_number(&self) -> u8 {
        match self {
            StickTopology::Linear => 0,
            StickTopology::Circular => 1
        }
    }
}

#[derive(Clone)]
pub struct Stick {
    id: StickId,
    size: u64,
    topology: StickTopology,
    tags: HashSet<String>
}

impl Stick {
    pub fn new(id: &StickId, size: u64, topology: StickTopology, tags: &[String],) -> Stick {
        Stick {
            id: id.clone(),
            size, topology,
            tags: tags.iter().cloned().collect(),
        }
    }

    pub fn get_id(&self) -> &StickId { &self.id }
    pub fn size(&self) -> u64 { self.size }
    pub fn tags(&self) -> &HashSet<String> { &self.tags }
    pub fn topology(&self) -> &StickTopology { &self.topology }
}