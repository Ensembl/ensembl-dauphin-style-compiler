use std::fmt;
use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use peregrine_toolkit::lock;
use peregrine_toolkit::serdetools::ByteData;
use peregrine_toolkit::serdetools::ForceString;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::MapAccess;
use serde::de::Visitor;
use crate::BackendNamespace;
use super::data::ReceivedData;

#[derive(serde_derive::Deserialize)]
#[serde(transparent)]
pub struct AssetsLoader {
    data: HashMap<String,Arc<Asset>>
}

#[derive(Clone)]
pub struct Assets {
    assets: Arc<Mutex<HashMap<(Option<BackendNamespace>,String),Arc<Asset>>>>
}

impl Assets {
    pub fn load(&mut self, loader: AssetsLoader, backend_namespace: Option<BackendNamespace>) {
        let mut assets = lock!(self.assets);
        for (name,asset) in loader.data {
            assets.insert((backend_namespace.clone(),name),asset);
        }
    }

    pub fn empty() -> Assets {
        Assets { assets: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn add(&mut self, assets: &Assets) {
        let mut self_assets = lock!(self.assets);
        for (key,value) in lock!(assets.assets).iter() {
            self_assets.insert(key.clone(),value.clone());
        }
    }

    pub fn get(&self, channel: Option<&BackendNamespace>, key: &str) -> Option<Arc<Asset>> {
        self.assets.lock().unwrap().get(&(channel.cloned(),key.to_string())).cloned()
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an Asset")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut bytes = None;
        let mut metadata = HashMap::new();
        while let Some(key) = access.next_key()? {
            match key {
                "data" => { 
                    let data : ByteData = access.next_value()?;
                    bytes = Some(ReceivedData::new_bytes(data.data));
                },
                key => {
                    let value : ForceString = access.next_value()?;
                    metadata.insert(key.to_string(),value.0);
                }
            }
        }
        Ok(Asset { bytes, metadata })
    }
}

impl<'de> Deserialize<'de> for Asset {
    fn deserialize<D>(deserializer: D) -> Result<Asset, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(AssetVisitor)
    }
}
