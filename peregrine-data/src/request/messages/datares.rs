use anyhow::anyhow as err;
use std::{collections::HashMap, fmt};
use serde::{Deserializer, de::Visitor};
use serde_derive::Deserialize;
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;

struct DatasetResponse(HashMap<String,ReceivedData>);

#[derive(Deserialize)]
pub struct DataResponse {
    data: DatasetResponse
}

impl DataResponse {
    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        for (name,data) in &self.data.0 {
            account_builder.add(name,data.len());
        }

    }
}

struct DatasetResponseVisitor;

impl<'de> Visitor<'de> for DatasetResponseVisitor {
    type Value = DatasetResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a data response") }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: serde::de::MapAccess<'de> {
        let mut data = HashMap::new();
        while let Some((k,v)) = map.next_entry::<String,ReceivedData>()? {
            data.insert(k,v);
        }
        Ok(DatasetResponse(data))
    }
}

impl<'de> serde::Deserialize<'de> for DatasetResponse {
    fn deserialize<D>(deserializer: D) -> Result<DatasetResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(DatasetResponseVisitor)
    }
}

impl DataResponse {
    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.0.get(name).ok_or_else(|| err!("no such data {}",name))
    }
}
