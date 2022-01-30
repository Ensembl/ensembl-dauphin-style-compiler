use std::collections::HashMap;

use minicbor::{Decoder, Decode, decode::Error};

use crate::core::error::EarpRuntimeError;

use super::earpfilereader::{AssetData, Operand, EarpFileReader};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct TopLevel {
    magic_got: Option<String>,
    entry_points: HashMap<String,i64>,
    assets: HashMap<String,AssetData>,
    sets: Vec<(String,u64,u64)>,
    instructions: Vec<(u64,Vec<Operand>)>
}

impl TopLevel {
    pub(super) fn into_earpfile(self) -> Result<EarpFileReader,EarpRuntimeError> {
        EarpFileReader::from_top_level(self.entry_points,self.assets)
    }
}

pub(crate) fn map_error<T>(input: Result<T,minicbor::decode::Error>) -> Result<T,EarpRuntimeError> {
    input.map_err(|e| EarpRuntimeError::BadEarpFile(e.to_string()))
}

fn extract_varieties(mut input: usize) -> Vec<usize> {
    let mut out = vec![];
    while input % 4 != 0 {
        out.push(input % 4);
        input /= 4;
    }
    out
}

fn cbor_iter<F,G>(decoder: &mut Decoder, mut cb: F, start: G) -> Result<(),minicbor::decode::Error> 
        where F: FnMut(&mut Decoder) -> Result<bool,minicbor::decode::Error>,
              G: FnOnce(&mut Decoder) -> Result<Option<u64>,minicbor::decode::Error> {
    if let Some(len) = start(decoder)? {
        for _ in 0..len {
            cb(decoder)?;
        }
    } else {
        while cb(decoder)? {}
    }
    Ok(())
}

fn cbor_array<F>(decoder: &mut Decoder, cb: F) -> Result<(),minicbor::decode::Error> 
        where F: FnMut(&mut Decoder) -> Result<bool,minicbor::decode::Error> {
    cbor_iter(decoder,cb,|decoder| decoder.array())
}

fn cbor_map<F>(decoder: &mut Decoder, cb: F) -> Result<(),minicbor::decode::Error> 
        where F: FnMut(&mut Decoder) -> Result<bool,minicbor::decode::Error> {
    cbor_iter(decoder,cb,|decoder| decoder.map())
}

impl<'b> Decode<'b> for TopLevel {
    fn decode(decoder: &mut Decoder<'b>) -> Result<Self, minicbor::decode::Error> {
        let mut magic_got = None;
        let mut entry_points = HashMap::new();
        let mut assets = HashMap::new();
        let mut sets = vec![];
        let mut instructions = vec![];
        decoder.map()?;
        loop {
            match decoder.str() {
                Ok("M") => {
                    magic_got = Some(decoder.str()?.to_string());
                },
                Ok("S") => {
                    cbor_array(decoder,|decoder| {
                        Ok(if let Ok(name) = decoder.str() {
                            let version = decoder.u64()?;
                            let offset = decoder.u64()?;
                            sets.push((name.to_string(),version,offset));
                            true
                        } else {
                            false
                        })
                    })?;
                },
                Ok("E") => {
                    cbor_map(decoder,|decoder| {
                        Ok(if let Ok(key) = decoder.str() {
                            let value = decoder.i64()?;
                            entry_points.insert(key.to_string(),value);
                            true
                        } else {
                            false
                        })
                    })?;
                },
                Ok("I") => {
                    cbor_array(decoder,|decoder| {
                        Ok(if let Ok(opcode) = decoder.u64() {
                            let mut operands = vec![];
                            for variety in extract_varieties(decoder.u64()? as usize) {
                                operands.push(Operand::decode(variety,decoder)?);
                            }
                            instructions.push((opcode,operands));
                            true
                        } else {
                            false
                        })
                    })?;
                },
                Ok("A") => {
                    cbor_map(decoder,|decoder| {
                        Ok(if let Ok(key) = decoder.str() {
                            let value = if decoder.probe().str().is_ok() {
                                AssetData::String(decoder.str()?.to_string())
                            } else {
                                AssetData::Bytes(decoder.bytes()?.to_vec())
                            };
                            assets.insert(key.to_string(),value);
                            true
                        } else {
                            false
                        })
                    })?;
                },
                Ok(_) => {
                    decoder.skip()?;
                },
                Err(_) => { break; }
            }
        }
        Ok(TopLevel {
            magic_got,
            entry_points,
            assets,
            sets,
            instructions
        })
    }
}
