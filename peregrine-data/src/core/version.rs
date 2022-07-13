use std::collections::BTreeMap;
use serde_cbor::Value as CborValue;

const BE_VERSION: u32 = 11;

#[derive(Clone)]
pub struct VersionMetadata {
    be_version: u32
}

impl VersionMetadata {
    pub fn new() -> VersionMetadata {
        VersionMetadata {
            be_version: BE_VERSION
        }
    }

    pub fn encode(&self) -> CborValue {
        let mut map = BTreeMap::new();
        map.insert(CborValue::Text("egs".to_string()),CborValue::Integer(self.be_version.into()));
        CborValue::Map(map)
    }

    pub fn backend_version(&self) -> u32 { self.be_version }
}