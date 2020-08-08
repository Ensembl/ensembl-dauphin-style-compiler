use anyhow;
use crate::command::{ Instruction, InstructionType };
use chrono::{ DateTime, Utc, offset::TimeZone };
use serde_cbor::Value as CborValue;
use dauphin_interp::command::ProgramMetadata;
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::cbor::{ cbor_make_map, cbor_map, cbor_string, cbor_option, cbor_int };
use std::convert::TryInto;

#[derive(Debug)]
pub struct ProgramMetadataBuilder {
    metadata: ProgramMetadata
}

impl ProgramMetadataBuilder {
    pub fn new(name: &str, note: Option<&str>, instrs: &[Instruction]) -> ProgramMetadataBuilder {
        let now : DateTime<Utc> = Utc::now();
        let user : Option<String> = users::get_current_username().and_then(|osstr| {
            osstr.into_string().ok()
        });        
        ProgramMetadataBuilder {
            metadata: ProgramMetadata::new(
                name,
                user.as_ref().map(|x| x as &str),
                now.timestamp_millis().into(),
                instrs.len() as u32,
                (instrs.iter().filter(|instr| if let InstructionType::Pause(_) = instr.itype { true } else { false }).count()+1) as u32,
                note
            )
        }
    }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        cbor_make_map(&vec![
            "name", "user", "ctime", "instr_count", "inter_count", "note"
        ],vec![
            CborValue::Text(self.name().to_string()),
            self.user().as_ref().map(|x| CborValue::Text(x.to_string())).unwrap_or(CborValue::Null),
            CborValue::Integer(self.metadata.ctime().into()),
            CborValue::Integer(self.instr_count().into()),
            CborValue::Integer(self.inter_count().into()),
            self.note().as_ref().map(|x| CborValue::Text(x.to_string())).unwrap_or(CborValue::Null)
        ])
    }

    pub fn deserialize(cbor: &CborValue) -> anyhow::Result<ProgramMetadataBuilder> {
        Ok(ProgramMetadataBuilder {
            metadata: ProgramMetadata::deserialize(cbor)?
        })
    }

    pub fn name(&self) -> &str { self.metadata.name() }
    pub fn ctime(&self) -> DateTime<Utc> { Utc.timestamp_millis(self.metadata.ctime()) }
    pub fn user(&self) -> Option<&str> { self.metadata.user().as_ref().map(|x| x as &str) }
    pub fn note(&self) -> Option<&str> { self.metadata.note().as_ref().map(|x| x as &str) }
    pub fn instr_count(&self) -> u32 { self.metadata.instr_count() }
    pub fn inter_count(&self) -> u32 { self.metadata.inter_count() }
}
