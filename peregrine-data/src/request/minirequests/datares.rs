use anyhow::anyhow as err;
use peregrine_toolkit::cbor::{cbor_into_drained_map };
use peregrine_toolkit::serdetools::{st_field, st_err, ByteData };
use serde::de::{Visitor, MapAccess, };
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::{collections::HashMap};
use crate::request::core::response::MiniResponseVariety;
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;
use inflate::inflate_bytes_zlib;

pub struct DataRes {
    data: HashMap<String,ReceivedData>,
    invariant: bool
}

impl DataRes {
    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        for (name,data) in &self.data {
            account_builder.add(name,data.len());
        }
    }

    #[cfg(debug_assertions)]
    pub fn keys(&self) -> Vec<String> { self.data.keys().cloned().collect::<Vec<_>>() }

    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.get(name).ok_or_else(|| err!("no such data {}: have {}",
            name,
            self.data.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    }

    pub(crate) fn is_invariant(&self) -> bool { self.invariant }
}

struct DataVisitor;

impl<'de> Visitor<'de> for DataVisitor {
    type Value = DataRes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a DataRes")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut data = None;
        let mut invariant = false;
        while let Some(key) = access.next_key()? {
            match key {
                "data" => { data = Some(access.next_value()?) },
                "invariant" => { invariant = access.next_value()? },
                _ => {}
            }
        }
        let value : ByteData = st_field("data",data)?;
        let bytes = st_err(inflate_bytes_zlib(&value.data),"uninflatable")?;
        let value = st_err(serde_cbor::from_slice(&bytes),"corrupt payload/A")?;
        let mut data = st_err(cbor_into_drained_map(value),"corrupt payload/B")?;
        let data = st_err(data.drain(..).map(|(k,v)| {
            Ok((k,ReceivedData::decode(v)?))
        }).collect::<Result<HashMap<String,ReceivedData>,String>>(),"corrupt payload/C")?;
        Ok(DataRes {
            data, 
            invariant
        })
    }
}

impl<'de> Deserialize<'de> for DataRes {
    fn deserialize<D>(deserializer: D) -> Result<DataRes, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(DataVisitor)
    }
}

impl MiniResponseVariety for DataRes {
    fn description(&self) -> &str { "data" }
    fn total_size(&self) -> usize { self.data.values().map(|x| x.len()).sum() }

    fn component_size(&self) -> Vec<(String,usize)> {
        self.data.iter().map(|(k,v)| (k.to_string(),v.len())).collect::<Vec<_>>()
    }
}
