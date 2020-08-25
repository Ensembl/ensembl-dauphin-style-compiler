use anyhow;
use dauphin_compile::command::{
    Command, CommandSchema, CommandType, CommandTrigger, PreImageOutcome, PreImagePrepare, CompLibRegister, Instruction, InstructionType
};
use dauphin_compile::model::PreImageContext;
use dauphin_interp::command::{ InterpCommand, CommandSetId, Identifier };
use dauphin_interp::types::{ RegisterSignature };
use dauphin_interp::runtime::{ Register };
use dauphin_interp::util::DauphinError;
use serde_cbor::Value as CborValue;
use crate::make_peregrine_interp;
use super::boot::AddStickAuthorityCommandType;

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xD6BF21A90B89A2CB)
}

pub(super) fn peregrine(name: &str) -> Identifier {
    Identifier::new("peregrine",name)
}

pub fn make_peregrine() -> CompLibRegister {
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("add_stick_authority",Some(0),AddStickAuthorityCommandType());
    set.add_header("peregrine",include_str!("header.dp"));
    set
}
