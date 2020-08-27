use anyhow;
use dauphin_compile::command::{
    CompLibRegister, Instruction, InstructionType
};
use dauphin_compile::model::PreImageContext;
use dauphin_interp::command::{ CommandSetId, Identifier };
use dauphin_interp::types::{ RegisterSignature };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;
use crate::make_peregrine_interp;
use super::boot::{ AddStickAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xE4F0C0276A75C1A9)
}

pub(super) fn peregrine(name: &str) -> Identifier {
    Identifier::new("peregrine",name)
}

pub fn make_peregrine() -> CompLibRegister {
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("add_stick_authority",Some(0),AddStickAuthorityCommandType());
    set.push("get_stick_id",Some(1),GetStickIdCommandType());
    set.push("get_stick_data",Some(2),GetStickDataCommandType());
    set.push("add_stick",Some(3),AddStickCommandType());
    set.add_header("peregrine",include_str!("header.dp"));
    set
}
