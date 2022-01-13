use dauphin_compile::command::{
    CompLibRegister
};
use dauphin_interp::command::{ CommandSetId };
use crate::compile::boot::AddJumpCommandType;
use crate::make_peregrine_interp;
use super::boot::{
    AddAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType,
    GetJumpDataCommandType, GetJumpLocationCommandType
};
use super::data::{ GetLaneCommandType, GetDataCommandType, DataStreamCommandType, OnlyWarmCommandType };
use super::decompress::{
    InflateBytesCommandType, InflateStringCommandType, Lesqlite2CommandType, ZigzagCommandType, DeltaCommandType,
    ClassifyCommandType, SplitStringCommandType, BaseFlipCommandType
};
use super::track::{ 
    NewLaneCommandType, AddTagCommandType, AddTrackCommandType, DataSourceCommandType, AddAllotmentCommandType,
    AddSwitchCommandType, SetSwitchCommandType, ClearSwitchCommandType, AppendGroupCommandType, AppendDepthCommandType
};
use super::geometry:: {
    PatinaFilledCommandType, PatinaHollowCommandType, DirectColourCommandType, ZMenuCommandType, PatinaZMenuCommandType, PenCommandType,
    PlotterCommandType, UseAllotmentCommandType, SpaceBaseCommandType, SimpleColourCommandType, StripedCommandType,
    BarCommandType, BpRangeCommandType, SpotColourCommandType
};
use super::shape::{ WiggleCommandType, RectangleCommandType, Text2CommandType, ImageCommandType };
use super::switch::{ GetSwitchCommandType, ListSwitchCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(44,0),0x1C43D4525BF8DBC4)
}

pub fn make_peregrine() -> CompLibRegister {
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("add_stick_authority",Some(0),AddAuthorityCommandType());
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
    set.push("barred",Some(37),BarCommandType());
    set.push("base_flip",Some(38),BaseFlipCommandType());
    set.push("add_jump",Some(39),AddJumpCommandType());
    set.push("get_jump_data",Some(40),GetJumpDataCommandType());
    set.push("get_jump_location",Some(41),GetJumpLocationCommandType());
    set.push("list_switch",Some(42),ListSwitchCommandType());
    set.push("only_warm",Some(43),OnlyWarmCommandType());
    set.push("draw_image",Some(44),ImageCommandType());
    set.push("bp_range",Some(45),BpRangeCommandType());
    set.push("spot_colour",Some(46),SpotColourCommandType());
    set.push("append_group",Some(47),AppendGroupCommandType());
    set.push("append_depth",Some(48),AppendDepthCommandType());
    set.add_header("peregrine",include_str!("header.egs"));
    set
}
