use std::collections::BTreeMap;

use crate::{Region, BackendNamespace, request::core::minirequest::MiniRequestVariety};
use serde::{Serialize, ser::SerializeSeq};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct DataRequest {
    channel: BackendNamespace,
    name: String,
    region: Region,
    scope: BTreeMap<String,Vec<String>>,
    accept: String
}

impl DataRequest {
    pub fn new(channel: &BackendNamespace, name: &str, region: &Region) -> DataRequest {
        DataRequest {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone(),
            scope: BTreeMap::new(),
            accept: "release".to_string()
        }
    }

    pub fn channel(&self) -> &BackendNamespace { &self.channel }
    pub fn name(&self) -> &str { &self.name }
    pub fn region(&self) -> &Region { &self.region }
    pub fn scope(&self) -> &BTreeMap<String,Vec<String>> { &self.scope }

    pub fn to_invariant(&self) -> DataRequest {
        let mut out = self.clone();
        out.region = out.region.to_invariant();
        out
    }

    pub fn add_scope(&self, key: &str, values: &[String]) -> DataRequest {
        let mut out = self.clone();
        out.scope.insert(key.to_string(),values.to_vec());
        out
    }
}

impl MiniRequestVariety for DataRequest {
    fn description(&self) -> String { "data".to_string() }
    fn opcode(&self) -> u8 { 4 }
}

impl Serialize for DataRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(5))?;
        seq.serialize_element(&self.channel)?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.region)?;
        seq.serialize_element(&self.scope)?;
        seq.serialize_element(&self.accept)?;
        seq.end()
    }
}
