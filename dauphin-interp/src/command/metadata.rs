use anyhow;
use serde_cbor::Value as CborValue;
use crate::util::DauphinError;
use crate::util::cbor::{ cbor_map, cbor_string, cbor_option, cbor_int };
use std::convert::TryInto;

#[derive(Debug)]
pub struct ProgramMetadata {
    name: String,
    user: Option<String>,
    ctime: i64,
    instr_count: u32,
    inter_count: u32,
    note: Option<String>
}

impl ProgramMetadata {
    pub fn new(name: &str, user: Option<&str>, ctime: i64, instr_count: u32, inter_count: u32, note: Option<&str>) -> ProgramMetadata {
        ProgramMetadata {
            name: name.to_string(),
            user: user.map(|x| x.to_string()),
            ctime, instr_count, inter_count,
            note: note.map(|x| x.to_string())
        }
    }

    pub fn deserialize(cbor: &CborValue) -> anyhow::Result<ProgramMetadata> {
        let data = cbor_map(cbor,&vec![
            "name", "user", "ctime", "instr_count", "inter_count", "note"
        ])?;
        Ok(ProgramMetadata {
            name: cbor_string(data[0])?,
            user: cbor_option(data[1],|x| cbor_string(x))?,
            ctime: cbor_int(data[2],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?,
            instr_count: cbor_int(data[3],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?,
            inter_count: cbor_int(data[4],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?,
            note: cbor_option(data[5],|x| cbor_string(x))?
        })
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn ctime(&self) -> i64 { self.ctime }
    pub fn user(&self) -> Option<&str> { self.user.as_ref().map(|x| x as &str) }
    pub fn note(&self) -> Option<&str> { self.note.as_ref().map(|x| x as &str) }
    pub fn instr_count(&self) -> u32 { self.instr_count }
    pub fn inter_count(&self) -> u32 { self.inter_count }
}
