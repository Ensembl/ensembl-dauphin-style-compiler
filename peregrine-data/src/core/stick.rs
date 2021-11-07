use anyhow::bail;
use peregrine_toolkit::serde::de_wrap;
use serde::{Deserializer, Serializer};
use serde_derive::Deserialize;
use std::collections::HashSet;
use std::fmt::{ self, Display, Formatter };
#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct StickId(String);

impl StickId {
    pub fn new(name: &str) -> StickId {
        StickId(name.to_string())
    }

    pub fn get_id(&self) -> &str { &self.0 }
}

impl serde::Serialize for StickId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl Display for StickId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
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

#[derive(Deserialize)]
struct StickResponse {
    id: String,
    size: u64,
    topology: u8,
    tags: Vec<String>
}

impl<'de> serde::Deserialize<'de> for Stick {
    fn deserialize<D>(deserializer: D) -> Result<Stick, D::Error> where D: Deserializer<'de> {
        let mut r = StickResponse::deserialize(deserializer)?;
        let topology = match r.topology {
            0 => StickTopology::Linear,
            1 => StickTopology::Circular,
            _ => de_wrap(Err("unknown topology"))?
        };
        let tags : HashSet<_> = r.tags.drain(..).collect();
        Ok(Stick { id: StickId::new(&r.id), size: r.size, topology, tags })
    }
}
