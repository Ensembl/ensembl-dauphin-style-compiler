use anyhow::bail;
use peregrine_toolkit::cbor::{cbor_as_number, cbor_as_str, cbor_into_map, cbor_into_vec, cbor_map_key, cbor_map_optional_key};
use std::collections::{BTreeMap, HashSet};
use std::fmt::{ self, Display, Formatter };
use serde_cbor::Value as CborValue;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct StickId(String);

impl StickId {
    pub fn new(name: &str) -> StickId {
        StickId(name.to_string())
    }

    pub fn get_id(&self) -> &str { &self.0 }

    pub fn encode(&self) -> CborValue {
        CborValue::Text(self.0.clone())
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
pub enum StickResponse {
    Stick(Stick),
    Unknown(String)
}

impl StickResponse {
    pub fn decode(value: CborValue) -> Result<StickResponse,String> {
        let mut map = cbor_into_map(value)?;
        Ok(if let Some(error) = cbor_map_optional_key(&mut map,"error") {
            StickResponse::Unknown(cbor_as_str(&error)?.to_string())
        } else {
            StickResponse::Stick(StickResponse::decode_stick(map)?)
        })
    }

    fn decode_stick(mut map: BTreeMap<CborValue,CborValue>) -> Result<Stick,String> {
        let mut tags = cbor_into_vec(cbor_map_key(&mut map,"tags")?)?;
        let tags = tags.drain(..).map(|x| cbor_as_str(&x).map(|x| x.to_string())).collect::<Result<HashSet<_>,_>>()?;
        let id = cbor_as_str(&cbor_map_key(&mut map,"id")?)?.to_string();
        let size = cbor_as_number(&cbor_map_key(&mut map,"size")?)?;
        let topology = cbor_as_number(&cbor_map_key(&mut map,"topology")?)?;
        let topology = match topology {
            0 => StickTopology::Linear,
            1 => StickTopology::Circular,
            x => { return Err(format!("unknown topology: {}",x)); }
        };
        Ok(Stick { id: StickId::new(&id), size, topology, tags })        
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
