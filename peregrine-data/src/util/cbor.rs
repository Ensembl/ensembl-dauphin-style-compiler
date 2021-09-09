use anyhow::{ self, anyhow as err, bail };
use serde_cbor::Value as CborValue;

pub fn cbor_array<'a>(cbor: &'a CborValue, len: usize, or_more: bool) -> anyhow::Result<&'a Vec<CborValue>> {
    match cbor {
        CborValue::Array(a) => {
            if a.len() == len || (a.len() >= len && or_more) {
                return Ok(a);
            }
        },
        _ => {}
    }
    Err(err!(format!("expected array len={:?} or_more={:?} got {:?}",len,or_more,cbor)))
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
    bail!("not an integer");
}

pub fn cbor_bytes(cbor: &CborValue) -> anyhow::Result<Vec<u8>> {
    match cbor {
        CborValue::Bytes(b) => {
            Ok(b.clone())
        }
        _ => bail!("not a string")
    }
}

pub fn cbor_string(cbor: &CborValue) -> anyhow::Result<String> {
    match cbor {
        CborValue::Text(x) => Ok(x.to_string()),
        _ => bail!("not a string")
    }
}

pub fn cbor_map<'a>(cbor: &'a CborValue, keys: &[&str]) -> anyhow::Result<Vec<&'a CborValue>> {
    let mut out = vec![];
    match cbor {
        CborValue::Map(m) => {
            for key in keys {
                out.push(m.get(&CborValue::Text(key.to_string())).ok_or_else(|| {
                    err!(format!("no such key {}",key))
                })?);
            }
        },
        _ => { bail!("expected map got {:?}",cbor) }
    }
    Ok(out)
}

pub fn cbor_map_key<'a>(cbor: &'a CborValue, key: &str) -> anyhow::Result<Option<&'a CborValue>> {
    match cbor {
        CborValue::Map(m) => {
            Ok(m.get(&CborValue::Text(key.to_string())))
        },
        _ => { bail!("expected map got {:?}",cbor) }
    }
}

pub fn cbor_map_iter(cbor: &CborValue) -> anyhow::Result<impl Iterator<Item=(&CborValue,&CborValue)>> {
    match cbor {
        CborValue::Map(m) => {
            Ok(m.iter())
        },
        _ => {
            bail!("expected map")
        }
    }
}

pub fn cbor_bool(cbor: &CborValue) -> anyhow::Result<bool> {
    match cbor {
        CborValue::Bool(x) => Ok(*x),
        _ => bail!("expected bool")
    }
}

pub fn cbor_coerce_string(cbor: &CborValue) -> anyhow::Result<String> {
    match cbor {
        CborValue::Text(t) => { Ok(t.to_string()) },
        CborValue::Integer(i) => { Ok(i.to_string()) },
        CborValue::Float(f) => { Ok(f.to_string()) },
        _ => { bail!("cannot convert to string") }
    }
}