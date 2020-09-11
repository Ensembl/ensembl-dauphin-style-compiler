use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };
use super::data::{ GetPanelDeserializer, GetDataDeserializer, DataStreamDeserializer };
use super::decompress::{ 
    InflateBytesDeserializer, InflateStringDeserializer, Lesqlite2Deserializer, ZigzagDeserializer, DeltaDeserializer,
    ClassifyDeserializer, SplitStringDeserializer
};

use super::panel::{ NewPanelDeserializer, AddTagDeserializer, AddTrackDeserializer, SetScaleDeserializer, DataSourceDeserializer };
use super::geometry::{
    IntervalDeserializer, ScreenStartPairDeserializer, ScreenEndPairDeserializer, ScreenSpanPairDeserializer, PositionDeserializer,
    ScreenStartDeserializer, ScreenEndDeserializer, PinStartDeserializer, PinCentreDeserializer, PinEndDeserializer,
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer
};
use super::shape::{
    Rectangle2Deserializer, Rectangle1Deserializer, TextDeserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0x1A2FBF1E27E50B97)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddStickAuthorityDeserializer());
    set.push(GetStickIdDeserializer());
    set.push(GetStickDataDeserializer());
    set.push(AddStickDeserializer());
    set.push(NewPanelDeserializer());
    set.push(AddTagDeserializer());
    set.push(AddTrackDeserializer());
    set.push(SetScaleDeserializer());
    set.push(DataSourceDeserializer());
    set.push(IntervalDeserializer());
    set.push(ScreenStartPairDeserializer());
    set.push(ScreenEndPairDeserializer());
    set.push(ScreenSpanPairDeserializer());
    set.push(PositionDeserializer());
    set.push(ScreenStartDeserializer());
    set.push(ScreenEndDeserializer());
    set.push(PinStartDeserializer());
    set.push(PinCentreDeserializer());
    set.push(PinEndDeserializer());
    set.push(Rectangle2Deserializer());
    set.push(Rectangle1Deserializer());
    set.push(GetPanelDeserializer());
    set.push(GetDataDeserializer());
    set.push(DataStreamDeserializer());
    set.push(InflateBytesDeserializer());
    set.push(InflateStringDeserializer());
    set.push(Lesqlite2Deserializer());
    set.push(ZigzagDeserializer());
    set.push(DeltaDeserializer());
    set.push(PatinaFilledDeserializer());
    set.push(ClassifyDeserializer());
    set.push(SplitStringDeserializer());
    set.push(PatinaHollowDeserializer());
    set.push(DirectColourDeserializer());
    set.push(ZMenuDeserializer());
    set.push(PatinaZMenuDeserializer());
    set.push(PenDeserializer());
    set.push(TextDeserializer());
    set
}
