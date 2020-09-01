use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };
use super::panel::{ NewPanelDeserializer, AddTagDeserializer, AddTrackDeserializer, SetScaleDeserializer, DataSourceDeserializer };
use super::geometry::{
    IntervalDeserializer, ScreenStartPairDeserializer, ScreenEndPairDeserializer, ScreenSpanPairDeserializer, PositionDeserializer,
    ScreenStartDeserializer, ScreenEndDeserializer, PinStartDeserializer, PinCentreDeserializer, PinEndDeserializer
};
use super::shape::{
    Rectangle2Deserializer, Rectangle1Deserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0x81B8D9C006008C3)
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
    set
}
