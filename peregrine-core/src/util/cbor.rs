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
    Err(err!(format!("expected map got {:?}",cbor)))
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
        _ => { err!("expected map") }
    }
    Ok(out)
}

pub fn cbor_map_iter(cbor: &CborValue) -> anyhow::Result<impl Iterator<Item=(&CborValue,&CborValue)>> {
    match cbor {
        CborValue::Map(m) => {
            Ok(m.iter())
        },
        _ => {
            bail!(err!("expected map"))
        }
    }
}
