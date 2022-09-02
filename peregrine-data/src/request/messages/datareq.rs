use std::collections::BTreeMap;

use crate::{Region, core::channel::Channel};
use serde_cbor::{Value as CborValue};

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct DataRequest {
    channel: Channel,
    name: String,
    region: Region,
    scope: BTreeMap<String,Vec<String>>,
    accept: String
}

fn encode_scope(input: &BTreeMap<String,Vec<String>>) -> CborValue {
    let mut output = BTreeMap::new();
    for (key,value) in input.iter() {
        let value_value = value.iter().map(|v| CborValue::Text(v.to_string())).collect::<Vec<_>>();
        output.insert(CborValue::Text(key.to_string()),CborValue::Array(value_value));
    }
    CborValue::Map(output)
}

impl DataRequest {
    pub fn new(channel: &Channel, name: &str, region: &Region) -> DataRequest {
        DataRequest {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone(),
            scope: BTreeMap::new(),
            accept: "release".to_string()
        }
    }

    pub fn channel(&self) -> &Channel { &self.channel }
    pub fn name(&self) -> &str { &self.name }
    pub fn region(&self) -> &Region { &self.region }

    pub fn to_index_invariant(&self) -> DataRequest {
        let mut out = self.clone();
        out.region = out.region.to_index_invariant();
        out
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            self.channel.encode(),
            CborValue::Text(self.name.to_string()),
            self.region.encode(),
            encode_scope(&self.scope),
            CborValue::Text(self.accept.to_string()),
        ])
    }

    pub fn add_scope(&self, key: &str, values: &[String]) -> DataRequest {
        let mut out = self.clone();
        out.scope.insert(key.to_string(),values.to_vec());
        out
    }
}
