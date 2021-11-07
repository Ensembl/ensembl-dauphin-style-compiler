use std::fmt;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use peregrine_toolkit::serde::de_wrap;
use serde::Deserializer;
use serde::de::Visitor;
use crate::request::data::ReceivedData;
use serde_cbor::Value as CborValue;

#[derive(Clone)]
pub struct Assets {
    assets: Arc<Mutex<HashMap<String,Arc<Asset>>>>
}

impl Assets {
    pub fn empty() -> Assets {
        Assets { assets: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn add(&mut self, assets: &Assets) {
        let mut self_assets = self.assets.lock().unwrap();
        for (key,value) in assets.assets.lock().unwrap().iter() {
            self_assets.insert(key.to_string(),value.clone());
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<Asset>> {
        self.assets.lock().unwrap().get(key).cloned()
    }
}

struct AssetsVisitor;

impl<'de> Visitor<'de> for AssetsVisitor {
    type Value = Assets;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"an asset") }

    fn visit_map<A>(self, mut map: A) -> Result<Assets,A::Error> where A: serde::de::MapAccess<'de> {
        let mut assets = HashMap::new();
        while let Some((key,asset)) = map.next_entry::<String,Asset>()? {
            assets.insert(key,Arc::new(asset));
        }
        Ok(Assets { assets: Arc::new(Mutex::new(assets)) })
    }
}

impl<'de> serde::Deserialize<'de> for Assets {
    fn deserialize<D>(deserializer: D) -> Result<Assets, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(AssetsVisitor)
    }
}

pub struct Asset {
    bytes: Option<ReceivedData>,
    metadata: HashMap<String,String>
}

impl Asset {
    pub fn bytes(&self) -> Option<ReceivedData> { self.bytes.as_ref().cloned() }
    pub fn metadata(&self, key: &str) -> Option<&str> { self.metadata.get(key).map(|x| x.as_str()) }
    pub fn metadata_u32(&self, key: &str) -> Option<u32> { self.metadata(key).map(|v| v.parse::<u32>().ok()).flatten() }
}

struct AssetVisitor;

impl<'de> Visitor<'de> for AssetVisitor {
    type Value = Asset;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"an asset") }

    fn visit_map<A>(self, mut map: A) -> Result<Asset,A::Error> where A: serde::de::MapAccess<'de> {
        let mut bytes = None;
        let mut metadata = HashMap::new();
        while let Some(key) = map.next_key::<String>()? {
            if key == "data" {
                bytes = Some(map.next_value()?)
            } else {
                let raw_value : CborValue = map.next_value()?;
                let value = match raw_value {
                    CborValue::Text(t) => { t.to_string() },
                    CborValue::Integer(i) => { i.to_string() },
                    CborValue::Float(f) => { f.to_string() },
                    _ => { return de_wrap(Err("metadata value cannot be converted to string"))?; }            
                };
                metadata.insert(key.to_string(),value);
            }
        }
        Ok(Asset { bytes, metadata })
    }
}

impl<'de> serde::Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Asset, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(AssetVisitor)
    }
}
