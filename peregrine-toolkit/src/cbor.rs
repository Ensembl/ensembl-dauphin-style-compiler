use std::{collections::BTreeMap, convert::TryFrom};
use serde_cbor::Value as CborValue;

pub fn cbor_into_vec(value: CborValue) -> Result<Vec<CborValue>,String> {
    match value {
        CborValue::Array(v) => Ok(v),
        _ => { return Err("wrong type, expected array".to_string()) }
    }
}

pub fn cbor_into_map(value: CborValue) -> Result<BTreeMap<CborValue,CborValue>,String> {
    match value {
        CborValue::Map(m) => Ok(m),
        _ => { return Err("wrong type, expected array".to_string()) }
    }
}

pub fn cbor_into_drained_map(value: CborValue) -> Result<Vec<(String,CborValue)>,String> {
    let mut map = cbor_into_map(value)?;
    let keys = map.keys().map(|x| Ok(cbor_as_str(x)?.to_string())).collect::<Result<Vec<_>,String>>()?;
    let mut out = vec![];
    for key in keys {
        let value = map.remove(&CborValue::Text(key.clone())).unwrap();
        out.push((key,value));
    }
    Ok(out)
}

pub fn cbor_into_bytes(value: CborValue) -> Result<Vec<u8>,String> {
   match value {
        CborValue::Bytes(m) => Ok(m),
        _ => { return Err("wrong type, expected bytes".to_string()) }
    }
}

pub fn cbor_map_key(value: &mut BTreeMap<CborValue,CborValue>, key: &str) -> Result<CborValue,String> {
    value.remove(&CborValue::Text(key.to_string())).ok_or(format!("missing key: {}",key))
}

pub fn cbor_map_contains(value: &BTreeMap<CborValue,CborValue>, key: &str) -> bool {
    value.contains_key(&CborValue::Text(key.to_string()))
}

pub fn cbor_map_optional_key(value: &mut BTreeMap<CborValue,CborValue>, key: &str) -> Option<CborValue> {
    value.remove(&CborValue::Text(key.to_string()))
}

pub fn cbor_as_str(value: &CborValue) -> Result<&str,String> {
    match value {
        CborValue::Text(t) => Ok(t),
        _ => { return Err("wrong type, expected string".to_string()) }
    }
}

pub fn cbor_as_number<T: TryFrom<i128>>(value: &CborValue) -> Result<T,String> {
    let v = match value {
        CborValue::Integer(t) => *t,
        _ => { return Err("wrong type, expected string".to_string()) }
    };
    T::try_from(v).map_err(|_| format!("doesn't fit into type"))
}

pub fn check_array_len<T>(data: &Vec<T>, len: usize) -> Result<(),String> {
    if data.len() == len {
        Ok(())
    } else {
        Err(format!("wrong length: expected {} got {}",len,data.len()))
    }
}

pub fn check_array_min_len<T>(data: &Vec<T>, len: usize) -> Result<(),String> {
    if data.len() >= len {
        Ok(())
    } else {
        Err(format!("wrong length: expected {} got {}",len,data.len()))
    }
}

pub fn cbor_force_into_string(value: CborValue) -> Result<String,String> {
    Ok(match value {
        CborValue::Text(t) => { t.to_string() },
        CborValue::Integer(i) => { i.to_string() },
        CborValue::Float(f) => { f.to_string() },
        _ => { return Err(format!("metadata value cannot be converted to string")); }            
    })
}

#[macro_export]
macro_rules! decompose_vec_do {
    ($input:ident, [] $($reversed:tt)*) => { 
        $(let $reversed = $input.pop().unwrap(); )*
    };

    ($input:ident, [$first:tt $($rest:tt)*] $($reversed:tt)*) => { 
        $crate::decompose_vec_do!($input,[$($rest)*] $first $($reversed)*)  // recursion
    };
}

#[macro_export]
macro_rules! decompose_vec {
    ($input:tt, $($args:tt),*) => {
        $crate::decompose_vec_do!($input,[$($args)*]);
    } 
}
