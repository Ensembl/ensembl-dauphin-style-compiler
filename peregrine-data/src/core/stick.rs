use peregrine_toolkit::error::Error;
use serde::Serialize;
use serde_derive::Deserialize;
use std::collections::HashSet;
use std::fmt::{ self, Display, Formatter };
use std::hash::Hash;

#[derive(Clone,Debug,Hash,PartialEq,Eq,Deserialize)]
#[serde(transparent)]
pub struct StickId(String);

impl StickId {
    pub fn new(name: &str) -> StickId {
        StickId(name.to_string())
    }

    pub fn get_id(&self) -> &str { &self.0 }
}

impl Serialize for StickId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl Display for StickId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

#[derive(Clone,Deserialize)]
#[cfg_attr(debug_assertions,derive(Debug))]
#[repr(u8)]
pub enum StickTopology {
    Linear = 0,
    Circular = 1
}

impl StickTopology {
    pub fn from_number(n: u8) -> Result<StickTopology,Error> {
        Ok(match n {
            0 => StickTopology::Linear,
            1 => StickTopology::Circular,
            _ => { return Err(Error::operr("unknown topology")); }
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
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Stick {
    id: StickId,
    size: u64,
    topology: StickTopology,
    tags: HashSet<String>
}

impl PartialEq for Stick {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Stick {}

impl Hash for Stick {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Stick {
    pub fn new(id: &StickId, size: u64, topology: StickTopology, tags: &[String]) -> Stick {
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
