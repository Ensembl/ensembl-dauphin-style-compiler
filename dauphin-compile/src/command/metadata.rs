use anyhow;
use crate::command::{ Instruction, InstructionType };
use chrono::{ DateTime, Utc, offset::TimeZone };
use serde_cbor::Value as CborValue;
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_make_map, cbor_map, cbor_string, cbor_option, cbor_int };
use std::convert::TryInto;

#[derive(Debug)]
pub struct ProgramMetadata {
    name: String,
    user: Option<String>,
    ctime: DateTime<Utc>,
    instr_count: u32,
    inter_count: u32,
    note: Option<String>
}

impl ProgramMetadata {
    pub fn new(name: &str, note: Option<&str>, instrs: &[Instruction]) -> ProgramMetadata {
        ProgramMetadata {
            name: name.to_string(),
            user: users::get_current_username().and_then(|s| s.into_string().ok()),
            ctime: Utc::now(),
            instr_count: instrs.len() as u32,
            inter_count: (instrs.iter().filter(|instr| if let InstructionType::Pause(_) = instr.itype { true } else { false }).count()+1) as u32,
            note: note.map(|x| x.to_string())
        }
    }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        cbor_make_map(&vec![
            "name", "user", "ctime", "instr_count", "inter_count", "note"
        ],vec![
            CborValue::Text(self.name.to_string()),
            self.user.as_ref().map(|x| CborValue::Text(x.to_string())).unwrap_or(CborValue::Null),
            CborValue::Integer(self.ctime.timestamp_millis().into()),
            CborValue::Integer(self.instr_count.into()),
            CborValue::Integer(self.inter_count.into()),
            self.note.as_ref().map(|x| CborValue::Text(x.to_string())).unwrap_or(CborValue::Null)
        ])
    }

    pub fn deserialize(cbor: &CborValue) -> anyhow::Result<ProgramMetadata> {
        let data = cbor_map(cbor,&vec![
            "name", "user", "ctime", "instr_count", "inter_count", "note"
        ])?;
        Ok(ProgramMetadata {
            name: cbor_string(data[0])?,
            user: cbor_option(data[1],|x| cbor_string(x))?,
            ctime: Utc.timestamp_millis(cbor_int(data[2],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?),
            instr_count: cbor_int(data[3],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?,
            inter_count: cbor_int(data[4],None)?.try_into().map_err(|_| DauphinError::malformed("bad number"))?,
            note: cbor_option(data[5],|x| cbor_string(x))?
        })
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn ctime(&self) -> &DateTime<Utc> { &self.ctime }
    pub fn user(&self) -> Option<&str> { self.user.as_ref().map(|x| x as &str) }
    pub fn note(&self) -> Option<&str> { self.note.as_ref().map(|x| x as &str) }
    pub fn instr_count(&self) -> u32 { self.instr_count }
    pub fn inter_count(&self) -> u32 { self.inter_count }
}
