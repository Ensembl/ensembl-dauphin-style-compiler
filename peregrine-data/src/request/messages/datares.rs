use anyhow::anyhow as err;
use peregrine_toolkit::cbor::{cbor_into_drained_map,cbor_into_bytes};
use std::{collections::HashMap};
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;
use serde_cbor::Value as CborValue;
use inflate::inflate_bytes_zlib;

pub struct DataRes {
    data: HashMap<String,ReceivedData>
}

impl DataRes {
    pub(crate) fn account(&self, account_builder: &PacketDatastreamMetricBuilder) {
        for (name,data) in &self.data {
            account_builder.add(name,data.len());
        }
    }

    #[cfg(debug_assertions)]
    pub fn keys(&self) -> Vec<String> { self.data.keys().cloned().collect::<Vec<_>>() }

    pub fn decode(value: CborValue) -> Result<DataRes,String> {
        for (key,value) in cbor_into_drained_map(value)?.drain(..) {
            if key == "data" {
                let bytes = inflate_bytes_zlib(&cbor_into_bytes(value)?).map_err(|e| "corrupted data payload")?;
                let value = serde_cbor::from_slice(&bytes).map_err(|e| "corrupted data payload")?;
                let data = cbor_into_drained_map(value)?.drain(..).map(|(k,v)| {
                    Ok((k,ReceivedData::decode(v)?))
                }).collect::<Result<_,String>>()?;
                return Ok(DataRes { data });
            }
        }
        return Err(format!("missing key 'data'"));
    }

    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.get(name).ok_or_else(|| err!("no such data {}: have {}",
            name,
            self.data.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    }
}
