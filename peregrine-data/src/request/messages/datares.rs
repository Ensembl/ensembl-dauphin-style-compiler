use anyhow::anyhow as err;
use dauphin_interp::util::cbor::cbor_bool;
use peregrine_toolkit::cbor::{cbor_into_drained_map,cbor_into_bytes };
use peregrine_toolkit::log;
use std::{collections::HashMap};
use crate::{metric::datastreammetric::PacketDatastreamMetricBuilder};
use crate::core::data::ReceivedData;
use serde_cbor::Value as CborValue;
use inflate::inflate_bytes_zlib;

#[cfg(debug_big_requests)]
use peregrine_toolkit::{ warn, cbor::{ cbor_as_map, cbor_as_bytes, cbor_map_optional_key_ref }};

#[cfg(debug_big_requests)]
const TOO_LARGE : usize = 10*1024;

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

    #[cfg(debug_big_requests)]
    pub(crate) fn result_size(value: &CborValue) -> Result<usize,String> {
        let data = cbor_as_map(value)?;
        let mut total = 0;
        let mut big_keys = vec![];
        if let Some(data) = cbor_map_optional_key_ref(data,"data") {
            let bytes = inflate_bytes_zlib(cbor_as_bytes(data)?)?;
            let data = serde_cbor::from_slice(&bytes).map_err(|e| format!("cannot deserialize"))?;
            for (key,value) in cbor_as_map(&data)?.iter() {
                let this_len = cbor_as_bytes(value)?.len();
                if this_len > TOO_LARGE/5 {
                    big_keys.push(format!("{:?} = {}",key,this_len));
                }
                total += this_len;
            }    
        }
        if total > TOO_LARGE {
            warn!("excessive single response size {}: {}",total,big_keys.join(", "));
        }
        Ok(total)
    }

    #[cfg(debug_assertions)]
    pub fn keys(&self) -> Vec<String> { self.data.keys().cloned().collect::<Vec<_>>() }

    pub fn decode(value: CborValue) -> Result<DataRes,String> {
        let mut data = cbor_into_drained_map(value)?;
        let mut invariant = false;
        let mut out = None;
        for (key,value) in data.drain(..) {
            if key == "data" {
                let bytes = inflate_bytes_zlib(&cbor_into_bytes(value)?).map_err(|e| "corrupted data payload")?;
                let value = serde_cbor::from_slice(&bytes).map_err(|_| "corrupted data payload")?;
                out = Some(cbor_into_drained_map(value)?.drain(..).map(|(k,v)| {
                    Ok((k,ReceivedData::decode(v)?))
                }).collect::<Result<_,String>>()?);
            } else if key == "invariant" {
                invariant = cbor_bool(&value).ok().unwrap_or(false);
            }
        }
        if let Some(data) = out {
            return Ok(DataRes { data, invariant });
        }
        return Err(format!("missing key 'data'"));
    }

    pub fn get(&self, name: &str) -> anyhow::Result<&ReceivedData> {
        self.data.get(name).ok_or_else(|| err!("no such data {}: have {}",
            name,
            self.data.keys().cloned().collect::<Vec<_>>().join(", ")
        ))
    }

    pub(crate) fn is_invariant(&self) -> bool { self.invariant }
}
