use dauphin_compile::command::{
    CompLibRegister
};
use dauphin_interp::command::{ CommandSetId };
use crate::make_peregrine_interp;
use super::boot::{
    AddAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType
};
use super::data::{ 
    GetLaneCommandType, GetDataCommandType, DataStreamCommandType, OnlyWarmCommandType,
    RequestCommandType, RequestScopeCommandType, MakeRegionCommandType
};
use super::decompress::{
    Lesqlite2CommandType, ZigzagCommandType, DeltaCommandType,
    ClassifyCommandType, SplitStringCommandType, BaseFlipCommandType
};
use super::eoes::{EoesVarNumberCommandType, EoesVarStringCommandType, EoesVarBooleanCommandType, EoesNullCommandType, EoesArrayCommandType, EoesPairCommandType, EoesObjectCommandType, EoesConditionCommandType, EoesGroupCommandType, EoesAllCommandType, EoesVarCommandType, EoesNumberCommandType, EoesStringCommandType, EoesBooleanCommandType, EoesLateCommandType};
use super::track::{ 
    NewLaneCommandType, AddTagCommandType, AddTrackCommandType, DataSourceCommandType,
    AddSwitchCommandType, SetSwitchCommandType, ClearSwitchCommandType, AppendGroupCommandType, AppendDepthCommandType
};
use super::geometry:: {
    PatinaFilledCommandType, PatinaHollowCommandType, DirectColourCommandType, ZMenuCommandType, PatinaZMenuCommandType, PenCommandType,
    PlotterCommandType, UseAllotmentCommandType, SpaceBaseCommandType, SimpleColourCommandType, StripedCommandType,
    BarCommandType, BpRangeCommandType, SpotColourCommandType, PpcCommandType, StyleCommandType, PatinaSwitchCommandType, PatinaMetadataCommandType, BackgroundCommandType
};
use super::shape::{ WiggleCommandType, RectangleCommandType, Text2CommandType, ImageCommandType, EmptyCommandType, RunningTextCommandType };
use super::switch::{ ListSwitchCommandType, SwitchStringCommandType, SwitchNumberCommandType, SwitchBooleanCommandType, SwitchNullCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(55,0),0x60A1CBEFA9EA11D5)
}

pub fn make_peregrine() -> CompLibRegister {
    // next is 76; 24, 25, 39, 40, 41 are unused
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
    set.push("make_request",Some(10),RequestCommandType());
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
    set.push("lesqlite2",Some(26),Lesqlite2CommandType());
    set.push("zigzag",Some(27),ZigzagCommandType());
    set.push("delta",Some(28),DeltaCommandType());
    set.push("patina_filled",Some(29),PatinaFilledCommandType());
    set.push("classify",Some(30),ClassifyCommandType());
    set.push("split_string",Some(31),SplitStringCommandType());
    set.push("switch_null",Some(32),SwitchNullCommandType());
    set.push("track_set_switch",Some(33),SetSwitchCommandType());
    set.push("track_clear_switch",Some(34),ClearSwitchCommandType());
    set.push("simple_colour",Some(35),SimpleColourCommandType());
    set.push("striped",Some(36),StripedCommandType());
    set.push("barred",Some(37),BarCommandType());
    set.push("base_flip",Some(38),BaseFlipCommandType());
    set.push("list_switch",Some(42),ListSwitchCommandType());
    set.push("only_warm",Some(43),OnlyWarmCommandType());
    set.push("draw_image",Some(44),ImageCommandType());
    set.push("bp_range",Some(45),BpRangeCommandType());
    set.push("spot_colour",Some(46),SpotColourCommandType());
    set.push("append_group",Some(47),AppendGroupCommandType());
    set.push("append_depth",Some(48),AppendDepthCommandType());
    set.push("px_per_carriage",Some(49),PpcCommandType());
    set.push("style",Some(50),StyleCommandType());
    set.push("patina_switch",Some(51),PatinaSwitchCommandType());
    set.push("request_scope",Some(52),RequestScopeCommandType());
    set.push("empty",Some(53),EmptyCommandType());
    set.push("patina_metadata",Some(54),PatinaMetadataCommandType());
    set.push("eoes_var_number",Some(55),EoesVarNumberCommandType());
    set.push("eoes_var_string",Some(56),EoesVarStringCommandType());
    set.push("eoes_var_boolean",Some(57),EoesVarBooleanCommandType());
    set.push("eoes_null",Some(58),EoesNullCommandType());
    set.push("eoes_array",Some(59),EoesArrayCommandType());
    set.push("eoes_pair",Some(60),EoesPairCommandType());
    set.push("eoes_object",Some(61),EoesObjectCommandType());
    set.push("eoes_condition",Some(62),EoesConditionCommandType());
    set.push("eoes_group",Some(63),EoesGroupCommandType());
    set.push("eoes_all",Some(64),EoesAllCommandType());
    set.push("eoes_var",Some(65),EoesVarCommandType());
    set.push("eoes_number",Some(66),EoesNumberCommandType());
    set.push("eoes_string",Some(67),EoesStringCommandType());
    set.push("eoes_boolean",Some(68),EoesBooleanCommandType());
    set.push("eoes_late",Some(69),EoesLateCommandType());
    set.push("background",Some(70),BackgroundCommandType());
    set.push("switch_string",Some(71),SwitchStringCommandType());
    set.push("switch_number",Some(72),SwitchNumberCommandType());
    set.push("switch_boolean",Some(73),SwitchBooleanCommandType());
    set.push("running_text",Some(74),RunningTextCommandType());
    set.push("make_region",Some(75),MakeRegionCommandType());
    set.add_header("peregrine",include_str!("header.egs"));
    set
}
