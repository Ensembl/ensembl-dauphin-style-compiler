use std::collections::HashMap;
use minicbor::{Decoder, Decode, decode::Error};

use crate::core::error::EarpRuntimeError;

use super::toplevel::{ TopLevel, map_error };

#[cfg_attr(debug_assertions,derive(Debug))]
pub enum AssetData {
    String(String),
    Bytes(Vec<u8>)
}

const MAGIC_NUMBER : &str = "EARP0";

#[derive(Clone,Debug,PartialEq)]
pub(crate) enum Operand {
    Register(usize),
    UpRegister(usize),
    String(String),
    Boolean(bool),
    Integer(i64),
    Float(f64),
}

impl Operand {
    pub(super) fn decode(variety: usize, decoder: &mut Decoder) -> Result<Operand,minicbor::decode::Error> {
        Ok(if decoder.probe().i64().is_ok() {
            let value = decoder.i64()?;
            match variety {
                1 => Operand::Register(value as usize),
                2 => Operand::UpRegister(value as usize),
                _ => Operand::Integer(value)
            }
        } else if decoder.probe().str().is_ok() {
            Operand::String(decoder.str()?.to_string())
        } else if decoder.probe().bool().is_ok() {
            Operand::Boolean(decoder.bool()?)
        } else if decoder.probe().f64().is_ok() {
            Operand::Float(decoder.f64()?)
        } else {
            return Err(Error::Message("unexpected operand"));
        })
    }
}

pub struct InstructionSetId(pub String, pub i64);

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct EarpFileReader {
    entry_points: HashMap<String,i64>,
    assets: HashMap<String,AssetData>
}

impl EarpFileReader {
    pub(super) fn from_top_level(entry_points: HashMap<String,i64>, assets: HashMap<String,AssetData>) -> Result<EarpFileReader,EarpRuntimeError> {
        Ok(EarpFileReader { entry_points, assets })
    }

    pub fn new(data: &[u8]) -> Result<EarpFileReader,EarpRuntimeError> {
        let mut decoder = Decoder::new(data);
        map_error(TopLevel::decode(&mut decoder))?.into_earpfile()
    }
}