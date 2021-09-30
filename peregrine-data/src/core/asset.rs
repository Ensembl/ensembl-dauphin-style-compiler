use std::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use dauphin_interp::util::cbor::{cbor_map_iter, cbor_string};
use crate::{ cbor_bytes, cbor_coerce_string};
use serde_cbor::Value as CborValue;

pub(crate) struct AssetsBuilder {
    assets: HashMap<String,Arc<Asset>>
}

impl AssetsBuilder {
    pub(crate) fn new() -> AssetsBuilder {
        AssetsBuilder {
            assets: HashMap::new()
        }
    }

    pub(crate) fn insert(&mut self, key: &str, value: Asset) {
        self.assets.insert(key.to_string(),Arc::new(value));
    }

    pub(crate) fn build(&mut self) -> Assets {
        Assets { assets: Arc::new(Mutex::new(self.assets.clone())) }
    }
}

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

pub struct Asset {
    bytes: Option<Arc<Vec<u8>>>,
    metadata: HashMap<String,String>
}

impl Asset {
    pub(crate) fn new(cbor: &CborValue) -> anyhow::Result<Asset> {
        let mut metadata = HashMap::new();
        let mut bytes = None;
        for (key,value) in cbor_map_iter(cbor)? {
            match cbor_string(key)?.as_str() {
                "data" => { bytes = Some(Arc::new(cbor_bytes(value)?)); },
                key => {
                    metadata.insert(key.to_string(),cbor_coerce_string(value)?);
                }
            }
        }
        Ok(Asset { bytes, metadata })
    }

    pub fn bytes(&self) -> Option<Arc<Vec<u8>>> { self.bytes.as_ref().map(|x| x.clone()) }
    pub fn metadata(&self, key: &str) -> Option<&str> { self.metadata.get(key).map(|x| x.as_str()) }
    pub fn metadata_u32(&self, key: &str) -> Option<u32> { self.metadata(key).map(|v| v.parse::<u32>().ok()).flatten() }
}
