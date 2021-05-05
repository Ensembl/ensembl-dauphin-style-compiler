use anyhow::bail;
use std::collections::HashSet;
use serde_cbor::Value as CborValue;
use std::fmt::{ self, Display, Formatter };
use crate::util::message::DataMessage;
use crate::{ Allotment };
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

#[derive(Clone,Debug)]
pub struct Stick {
    id: StickId,
    size: u64,
    topology: StickTopology,
    tags: HashSet<String>,
    allotments: Vec<Allotment>
}

impl Stick {
    pub fn new(id: &StickId, size: u64, topology: StickTopology, tags: &[String], allotments: &[Allotment]) -> Stick {
        use web_sys::console;
        let a : Vec<_> = allotments.iter().map(|x| format!("{:?}",x)).collect();
        console::log_1(&format!("allotments {:?}",a).into());
        Stick {
            id: id.clone(),
            size, topology,
            tags: tags.iter().cloned().collect(),
            allotments: allotments.to_vec()
        }
    }

    pub fn get_id(&self) -> &StickId { &self.id }
    pub fn size(&self) -> u64 { self.size }
    pub fn tags(&self) -> &HashSet<String> { &self.tags }
    pub fn topology(&self) -> &StickTopology { &self.topology }
    pub fn allotments(&self) -> &[Allotment] { &self.allotments }
}