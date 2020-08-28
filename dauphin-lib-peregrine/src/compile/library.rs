use anyhow;
use dauphin_compile::command::{
    CompLibRegister, Instruction, InstructionType
};
use dauphin_interp::command::{ CommandSetId, Identifier };
use crate::make_peregrine_interp;
use super::boot::{ AddStickAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType };
use super::panel::{ NewPanelCommandType, AddTagCommandType, AddTrackCommandType, SetScaleCommandType, DataSourceCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0x748A24A0A2D68971)
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
    set.push("panel_new",Some(4),NewPanelCommandType());
    set.push("panel_add_tag",Some(5),AddTagCommandType());
    set.push("panel_add_track",Some(6),AddTrackCommandType());
    set.push("panel_set_scale",Some(7),SetScaleCommandType());
    set.push("panel_apply",Some(8),DataSourceCommandType());
    set.add_header("peregrine",include_str!("header.dp"));
    set
}
