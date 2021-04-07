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
use super::lane::{ 
    NewLaneCommandType, AddTagCommandType, AddTrackCommandType, SetScaleCommandType, DataSourceCommandType,
    SetMaxScaleJumpCommandType
};
use super::geometry:: {
    IntervalCommandType, ScreenStartPairCommandType, ScreenEndPairCommandType, ScreenSpanPairCommandType, PositionCommandType,
    ScreenStartCommandType, ScreenEndCommandType, PinStartCommandType, PinCentreCommandType, PinEndCommandType,
    PatinaFilledCommandType, PatinaHollowCommandType, DirectColourCommandType, ZMenuCommandType, PatinaZMenuCommandType, PenCommandType,
    PlotterCommandType
};
use super::shape::{ Rectangle2CommandType, Rectangle1CommandType, TextCommandType, WiggleCommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(1,0),0x20905D9CE1E9207C)
}

pub fn make_peregrine() -> CompLibRegister {
    let mut set = CompLibRegister::new(&peregrine_id(),Some(make_peregrine_interp()));
    set.push("add_stick_authority",Some(0),AddStickAuthorityCommandType());
    set.push("get_stick_id",Some(1),GetStickIdCommandType());
    set.push("get_stick_data",Some(2),GetStickDataCommandType());
    set.push("add_stick",Some(3),AddStickCommandType());
    set.push("lane_new",Some(4),NewLaneCommandType());
    set.push("lane_add_tag",Some(5),AddTagCommandType());
    set.push("lane_add_track",Some(6),AddTrackCommandType());
    set.push("lane_set_scale",Some(7),SetScaleCommandType());
    set.push("lane_apply",Some(8),DataSourceCommandType());
    set.push("interval",Some(9),IntervalCommandType());
    set.push("screen_start_pair",Some(10),ScreenStartPairCommandType());
    set.push("screen_end_pair",Some(11),ScreenEndPairCommandType());
    set.push("screen_span_pair",Some(12),ScreenSpanPairCommandType());
    set.push("position",Some(13),PositionCommandType());
    set.push("screen_start",Some(14),ScreenStartCommandType());
    set.push("screen_end",Some(15),ScreenEndCommandType());
    set.push("pin_start",Some(16),PinStartCommandType());
    set.push("pin_centre",Some(17),PinCentreCommandType());
    set.push("pin_end",Some(18),PinEndCommandType());
    set.push("rectangle2",Some(19),Rectangle2CommandType());
    set.push("rectangle1",Some(20),Rectangle1CommandType());
    set.push("get_lane",Some(21),GetLaneCommandType());
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
    set.push("patina_hollow",Some(32),PatinaHollowCommandType());
    set.push("colour",Some(33),DirectColourCommandType());
    set.push("zmenu",Some(34),ZMenuCommandType());
    set.push("patina_zmenu",Some(35),PatinaZMenuCommandType());
    set.push("pen",Some(36),PenCommandType());
    set.push("text",Some(37),TextCommandType());
    set.push("plotter",Some(38),PlotterCommandType());
    set.push("wiggle",Some(39),WiggleCommandType());
    set.push("lane_set_max_scale_jump",Some(40),SetMaxScaleJumpCommandType());
    set.add_header("peregrine",include_str!("header.egs"));
    set
}
