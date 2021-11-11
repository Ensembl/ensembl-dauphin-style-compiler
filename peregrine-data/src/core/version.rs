use std::collections::BTreeMap;
use serde_cbor::Value as CborValue;

const EGS_VERSION: u32 = 4;

#[derive(Clone)]
pub struct VersionMetadata {
    egs_version: u32
}

impl VersionMetadata {
    pub fn new() -> VersionMetadata {
        VersionMetadata {
            egs_version: EGS_VERSION
        }
    }

    pub fn encode(&self) -> CborValue {
        let mut map = BTreeMap::new();
        map.insert(CborValue::Text("egs".to_string()),CborValue::Integer(self.egs_version.into()));
        CborValue::Map(map)
    }
}