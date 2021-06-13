use dauphin_compile::command::{
    CompLibRegister
};
use dauphin_interp::command::{ CommandSetId };
use crate::make_peregrine_interp;
use super::boot::{ AddStickAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType };
use super::data::{ GetLaneCommandType, GetDataCommandType, DataStreamCommandType };
use super::decompress::{
    InflateBytesCommandType, InflateStringCommandType, Lesqlite2CommandType, ZigzagCommandType, DeltaCommandType,
    ClassifyCommandType, SplitStringCommandType
};
use super::track::{ 
    NewLaneCommandType, AddTagCommandType, AddTrackCommandType, DataSourceCommandType, AddAllotmentCommandType,
    AddSwitchCommandType, SetSwitchCommandType, ClearSwitchCommandType
};
use super::geometry:: {
    PatinaFilledCommandType, PatinaHollowCommandType, DirectColourCommandType, ZMenuCommandType, PatinaZMenuCommandType, PenCommandType,
    PlotterCommandType, UseAllotmentCommandType, SpaceBaseCommandType, SimpleColourCommandType, StripedCommandType
};
use super::shape::{ WiggleCommandType, RectangleCommandType, Text2CommandType };
use super::switch::GetSwitchCommandType;

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(24,0),0xA82453E40D434D10)
}

pub fn make_peregrine() -> CompLibRegister {
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("add_stick_authority",Some(0),AddStickAuthorityCommandType());
    set.push("get_stick_id",Some(1),GetStickIdCommandType());
    set.push("get_stick_data",Some(2),GetStickDataCommandType());
    set.push("add_stick",Some(3),AddStickCommandType());
    set.push("track_new",Some(4),NewLaneCommandType());
    set.push("track_add_tag",Some(5),AddTagCommandType());
    set.push("track_add_trigger",Some(6),AddTrackCommandType());
    set.push("wiggle",Some(7),WiggleCommandType());
    set.push("track_apply",Some(8),DataSourceCommandType());
    set.push("patina_hollow",Some(9),PatinaHollowCommandType());
    set.push("track_add_allotment",Some(10),AddAllotmentCommandType());
    set.push("track_add_switch",Some(11),AddSwitchCommandType());
    set.push("use_allotment",Some(12),UseAllotmentCommandType());
    set.push("direct_colour",Some(13),DirectColourCommandType());
    set.push("zmenu",Some(14),ZMenuCommandType());
    set.push("patina_zmenu",Some(15),PatinaZMenuCommandType());
    set.push("pen",Some(16),PenCommandType());
    set.push("spacebase",Some(17),SpaceBaseCommandType());
    set.push("plotter",Some(18),PlotterCommandType());
    set.push("text2",Some(19),Text2CommandType());
    set.push("rectangle",Some(20),RectangleCommandType());
    set.push("get_region",Some(21),GetLaneCommandType());
    set.push("get_data",Some(22),GetDataCommandType());
    set.push("data_stream",Some(23),DataStreamCommandType());
    set.push("inflate_bytes",Some(24),InflateBytesCommandType());
    set.push("inflate_string",Some(25),InflateStringCommandType());
    set.push("lesqlite2",Some(26),Lesqlite2CommandType());
    set.push("zigzag",Some(27),ZigzagCommandType());
    set.push("delta",Some(28),DeltaCommandType());
    set.push("patina_filled",Some(29),PatinaFilledCommandType());
    set.push("classify",Some(30),ClassifyCommandType());
    set.push("split_string",Some(31),SplitStringCommandType());
    set.push("get_switch",Some(32),GetSwitchCommandType());
    set.push("track_set_switch",Some(33),SetSwitchCommandType());
    set.push("track_clear_switch",Some(34),ClearSwitchCommandType());
    set.push("simple_colour",Some(35),SimpleColourCommandType());
    set.push("striped",Some(36),StripedCommandType());
    set.add_header("peregrine",include_str!("header.egs"));
    set
}
