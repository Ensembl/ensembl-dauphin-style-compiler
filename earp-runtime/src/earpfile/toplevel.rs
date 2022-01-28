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
                    decoder.array()?;
                    while let Ok(name) = decoder.str() {
                        let version = decoder.u64()?;
                        let offset = decoder.u64()?;
                        sets.push((name.to_string(),version,offset));
                    }
                },
                Ok("E") => {
                    decoder.map()?;
                    while let Ok(key) = decoder.str() {
                        let value = decoder.i64()?;
                        entry_points.insert(key.to_string(),value);
                    }
                },
                Ok("I") => {
                    decoder.array()?;
                    while let Ok(opcode) = decoder.u64() {
                        let mut operands = vec![];
                        for variety in extract_varieties(decoder.u64()? as usize) {
                            operands.push(Operand::decode(variety,decoder)?);
                        }
                        instructions.push((opcode,operands));
                    }
                },
                Ok("A") => {
                    decoder.map()?;
                    while let Ok(key) = decoder.str() {
                        let value = if decoder.probe().str().is_ok() {
                            AssetData::String(decoder.str()?.to_string())
                        } else {
                            AssetData::Bytes(decoder.bytes()?.to_vec())
                        };
                        assets.insert(key.to_string(),value);
                    }
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
