use dauphin_compile::command::{
    CompLibRegister
};
use dauphin_interp::command::{ CommandSetId };
use crate::make_peregrine_interp;
use super::boot::{ AddStickAuthorityCommandType, GetStickIdCommandType, GetStickDataCommandType, AddStickCommandType };
use super::data::{ GetPanelCommandType, GetDataCommandType, DataStreamCommandType };
use super::decompress::{
    InflateBytesCommandType, InflateStringCommandType, Lesqlite2CommandType, ZigzagCommandType, DeltaCommandType,
    ClassifyCommandType, SplitStringCommandType
};
use super::panel::{ NewPanelCommandType, AddTagCommandType, AddTrackCommandType, SetScaleCommandType, DataSourceCommandType };
use super::geometry:: {
    IntervalCommandType, ScreenStartPairCommandType, ScreenEndPairCommandType, ScreenSpanPairCommandType, PositionCommandType,
    ScreenStartCommandType, ScreenEndCommandType, PinStartCommandType, PinCentreCommandType, PinEndCommandType
};
use super::shape::{ Rectangle2CommandType, Rectangle1CommandType };

pub fn peregrine_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xEAE1F98BA1603375)
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
    set.push("get_panel",Some(21),GetPanelCommandType());
    set.push("get_data",Some(22),GetDataCommandType());
    set.push("data_stream",Some(23),DataStreamCommandType());
    set.push("inflate_bytes",Some(24),InflateBytesCommandType());
    set.push("inflate_string",Some(25),InflateStringCommandType());
    set.push("lesqlite2",Some(26),Lesqlite2CommandType());
    set.push("zigzag",Some(27),ZigzagCommandType());
    set.push("delta",Some(28),DeltaCommandType());
    // 29 is unused
    set.push("classify",Some(30),ClassifyCommandType());
    set.push("split_string",Some(31),SplitStringCommandType());
    set.add_header("peregrine",include_str!("header.dp"));
    set
}
