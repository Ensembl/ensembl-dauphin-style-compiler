/* 
 *  See the NOTICE file distributed with this work for additional information
 *  regarding copyright ownership.
 *  
 *  Licensed under the Apache License, Version 2.0 (the "License"); you may 
 *  not use this file except in compliance with the License. You may obtain a
 *  copy of the License at http://www.apache.org/licenses/LICENSE-2.0
 *  
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS, WITHOUT 
 *  WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use anyhow::{ self, bail, Context };
use crate::util::DauphinError;
use std::collections::BTreeMap;
use serde_cbor::Value as CborValue;

#[derive(Debug,PartialEq)]
pub enum CborType {
    Integer,
    Bool,
    Text,
    Map,
    Tag,
    Array,
    Null,
    Float,
    Bytes
}

pub fn cbor_serialize(program: &CborValue) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();
    serde_cbor::to_writer(&mut buffer,&program)
        .map_err(|x| DauphinError::malformed(&format!("cannot serialize cbor: {}",x.to_string())));
    Ok(buffer)
}

pub fn cbor_make_map(keys: &[&str], mut values: Vec<CborValue>) -> anyhow::Result<CborValue> {
    if keys.len() != values.len() {
        bail!(DauphinError::internal(file!(),line!()));
    }
    let mut out = BTreeMap::new();
    for (i,v) in values.drain(..).enumerate() {
        out.insert(CborValue::Text(keys[i].to_string()),v);
    }
    Ok(CborValue::Map(out))
}

pub fn cbor_update_map(data: &mut CborValue, key: &str, value: CborValue) -> anyhow::Result<()> {
    if let CborValue::Map(ref mut m) = data {
        m.insert(CborValue::Text(key.to_string()),value);
        Ok(())
    } else {
        Err(DauphinError::internal(file!(),line!()))
    }
}

pub fn cbor_type(cbor: &CborValue, allowed: Option<&[CborType]>) -> anyhow::Result<CborType> {
    let out = match cbor {
        CborValue::Integer(_) => CborType::Integer,
        CborValue::Bool(_) => CborType::Bool,
        CborValue::Text(_) => CborType::Text,
        CborValue::Map(_) => CborType::Map,
        CborValue::Tag(_,_) => CborType::Tag,
        CborValue::Array(_) => CborType::Array,
        CborValue::Null => CborType::Null,
        CborValue::Float(_) => CborType::Float,
        CborValue::Bytes(_) => CborType::Bytes,
        _ => { bail!(DauphinError::internal(file!(),line!())); }
    };
    if let Some(allowed) = allowed {
        if !allowed.contains(&out) {
            bail!(DauphinError::internal(file!(),line!()));
        }
    }
    Ok(out)
}

pub fn cbor_option<F,T>(cbor: &CborValue, cb: F) -> anyhow::Result<Option<T>> where F: FnOnce(&CborValue) -> anyhow::Result<T> {
    Ok(if let CborValue::Null = cbor { None } else { Some(cb(cbor)?) })
}

pub fn cbor_int(cbor: &CborValue, max: Option<i128>) -> anyhow::Result<i128>  {
    match cbor {
        CborValue::Integer(x) => {
            if let Some(max) = max {
                if *x >= 0 && *x <= max { return Ok(*x); }
            } else {
                return Ok(*x);
            }
        },
        _ => {}
    }
    bail!(DauphinError::internal(file!(),line!()));
}

pub fn cbor_float(cbor: &CborValue) -> anyhow::Result<f64>  {
    match cbor {
        CborValue::Float(x) => {
            return Ok(*x);
        },
        _ => {}
    }
    bail!(DauphinError::internal(file!(),line!()));
}

pub fn cbor_bool(cbor: &CborValue) -> anyhow::Result<bool> {
    match cbor {
        CborValue::Bool(x) => Ok(*x),
        _ => bail!(DauphinError::internal(file!(),line!()))
    }
}

pub fn cbor_string(cbor: &CborValue) -> anyhow::Result<String> {
    match cbor {
        CborValue::Text(x) => Ok(x.to_string()),
        _ => bail!(DauphinError::internal(file!(),line!()))
    }
}

pub fn cbor_map<'a>(cbor: &'a CborValue, keys: &[&str]) -> anyhow::Result<Vec<&'a CborValue>> {
    let mut out = vec![];
    match cbor {
        CborValue::Map(m) => {
            for key in keys {
                out.push(m.get(&CborValue::Text(key.to_string())).ok_or_else(|| {
                    DauphinError::internal(file!(),line!()).context(format!("key {}",key))
                })?);
            }
        },
        _ => { bail!(DauphinError::internal(file!(),line!())); }
    }
    Ok(out)
}

pub fn cbor_take_map(cbor: CborValue) -> anyhow::Result<BTreeMap<CborValue,CborValue>> {
    match cbor {
        CborValue::Map(m) => Ok(m),
        _ => Err(DauphinError::internal(file!(),line!()))
    }
}

pub fn cbor_map_iter(cbor: &CborValue) -> anyhow::Result<impl Iterator<Item=(&CborValue,&CborValue)>> {
    match cbor {
        CborValue::Map(m) => {
            Ok(m.iter())
        },
        _ => {
            bail!(DauphinError::internal(file!(),line!()));
        }
    }
}

pub fn cbor_map_iter_mut(cbor: &mut CborValue) -> anyhow::Result<impl Iterator<Item=(&CborValue,&mut CborValue)>> {
    match cbor {
        CborValue::Map(m) => {
            Ok(m.iter_mut())
        },
        _ => {
            bail!(DauphinError::internal(file!(),line!()));
        }
    }
}

pub fn cbor_entry<'a>(cbor: &'a CborValue, key: &str) -> anyhow::Result<Option<&'a CborValue>> {
    Ok(match cbor {
        CborValue::Map(m) => m.get(&CborValue::Text(key.to_string())),
        _ => { 
            return Err(DauphinError::internal(file!(),line!()).context(format!("expected map got {:?}",cbor)));
        }
    })
}

pub fn cbor_array<'a>(cbor: &'a CborValue, len: usize, or_more: bool) -> anyhow::Result<&'a Vec<CborValue>> {
    match cbor {
        CborValue::Array(a) => {
            if a.len() == len || (a.len() >= len && or_more) {
                return Ok(a);
            }
        },
        _ => {}
    }
    Err(DauphinError::malformed(&format!("expected map got {:?}",cbor)))
}