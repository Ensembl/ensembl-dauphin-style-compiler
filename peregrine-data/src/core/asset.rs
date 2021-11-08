use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use peregrine_toolkit::cbor::cbor_force_into_string;
use peregrine_toolkit::cbor::cbor_into_drained_map;
use serde_cbor::Value as CborValue;
use super::data::ReceivedData;

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

    pub fn decode(value: CborValue) -> Result<Assets,String> {
        let assets = cbor_into_drained_map(value)?.drain(..)
            .map(|(k,v)| Ok((k,Arc::new(Asset::decode(v)?))) )
            .collect::<Result<HashMap<_,_>,String>>()?;
        Ok(Assets { assets: Arc::new(Mutex::new(assets)) })
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

    pub fn decode(value: CborValue) -> Result<Asset,String> {
        let mut bytes = None;
        let mut metadata = HashMap::new();
        for (key,value) in cbor_into_drained_map(value)? {
            if key == "data" {
                bytes = Some(ReceivedData::decode(value)?);
            } else {
                let value = cbor_force_into_string(value)?;
                metadata.insert(key.to_string(),value);
            }
        }
        Ok(Asset { bytes, metadata })
    }
}
