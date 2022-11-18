use dauphin_compile::command::{
    CompLibRegister
};
use dauphin_interp::command::{ CommandSetId };
use crate::make_peregrine_interp;
use super::data::{ 
    GetLaneCommandType, GetDataCommandType, OnlyWarmCommandType,
    RequestCommandType, RequestScopeCommandType, MakeRegionCommandType, DataNumberCommandType, DataStringCommandType, DataBooleanCommandType
};
use super::decompress::{
    SplitStringCommandType, BaseFlipCommandType
};
use super::eoes::{EoesVarNumberCommandType, EoesVarStringCommandType, EoesVarBooleanCommandType, EoesNullCommandType, EoesArrayCommandType, EoesPairCommandType, EoesObjectCommandType, EoesConditionCommandType, EoesGroupCommandType, EoesAllCommandType, EoesVarCommandType, EoesNumberCommandType, EoesStringCommandType, EoesBooleanCommandType, EoesLateCommandType};
use super::track::{ 
    AppendGroupCommandType, AppendDepthCommandType
};
use super::geometry:: {
    PatinaFilledCommandType, PatinaHollowCommandType, DirectColourCommandType, ZMenuCommandType, PatinaZMenuCommandType, PenCommandType,
    PlotterCommandType, UseAllotmentCommandType, SpaceBaseCommandType, SimpleColourCommandType, StripedCommandType,
    BarCommandType, BpRangeCommandType, SpotColourCommandType, PpcCommandType, StyleCommandType, PatinaMetadataCommandType, BackgroundCommandType, PatinaSettingSetCommandType, PatinaSettingMemberCommandType, PatinaSpecialZoneCommandType
};
use super::shape::{ WiggleCommandType, RectangleCommandType, Text2CommandType, ImageCommandType, EmptyCommandType, RunningTextCommandType };
use super::switch::{ SettingBooleanCommandType, SettingNullCommandType, SettingNumberCommandType, SettingStringCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(64,0),0xA28D7DA9B2C26AB1)
}

pub fn make_peregrine() -> CompLibRegister {
    // next is 76; 24-28, 30, 32-34, 39-42, 51, 71-73 are unused
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("setting_string",Some(0),SettingStringCommandType());
    set.push("setting_number",Some(1),SettingNumberCommandType());
    set.push("setting_boolean",Some(2),SettingBooleanCommandType());
    set.push("setting_null",Some(3),SettingNullCommandType());
    set.push("patina_setting_set",Some(4),PatinaSettingSetCommandType());
    set.push("patina_setting_member",Some(5),PatinaSettingMemberCommandType());
    set.push("data_number",Some(6),DataNumberCommandType());
    set.push("wiggle",Some(7),WiggleCommandType());
    set.push("data_string",Some(8),DataStringCommandType());
    set.push("patina_hollow",Some(9),PatinaHollowCommandType());
    set.push("make_request",Some(10),RequestCommandType());
    set.push("data_string",Some(11),DataBooleanCommandType());
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
    set.push("patina_special_zone",Some(23),PatinaSpecialZoneCommandType());
    set.push("patina_filled",Some(29),PatinaFilledCommandType());
    set.push("split_string",Some(31),SplitStringCommandType());
    set.push("simple_colour",Some(35),SimpleColourCommandType());
    set.push("striped",Some(36),StripedCommandType());
    set.push("barred",Some(37),BarCommandType());
    set.push("base_flip",Some(38),BaseFlipCommandType());
    set.push("only_warm",Some(43),OnlyWarmCommandType());
    set.push("draw_image",Some(44),ImageCommandType());
    set.push("bp_range",Some(45),BpRangeCommandType());
    set.push("spot_colour",Some(46),SpotColourCommandType());
    set.push("append_group",Some(47),AppendGroupCommandType());
    set.push("append_depth",Some(48),AppendDepthCommandType());
    set.push("px_per_carriage",Some(49),PpcCommandType());
    set.push("style",Some(50),StyleCommandType());
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
    set.push("running_text",Some(74),RunningTextCommandType());
    set.push("make_region",Some(75),MakeRegionCommandType());
    set.add_header("peregrine",include_str!("header.egs"));
    set
}
