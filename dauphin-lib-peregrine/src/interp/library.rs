use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };
use super::data::{ GetLaneDeserializer, GetDataDeserializer, DataStreamDeserializer };
use super::decompress::{ 
    InflateBytesDeserializer, InflateStringDeserializer, Lesqlite2Deserializer, ZigzagDeserializer, DeltaDeserializer,
    ClassifyDeserializer, SplitStringDeserializer
};

use super::track::{ 
    NewLaneDeserializer, AddTagDeserializer, AddTriggerDeserializer, DataSourceDeserializer, AddSwitchDeserializer,
    AddAllotmentDeserializer
};
use super::geometry::{
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer, PlotterDeserializer, UseAllotmentDeserializer, SpaceBaseDeserializer
};
use super::shape::{
    WiggleDeserializer, RectangleDeserializer, Text2Deserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(20,0),0xC5A8F6011ED81ABF)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddStickAuthorityDeserializer());
    set.push(GetStickIdDeserializer());
    set.push(GetStickDataDeserializer());
    set.push(AddStickDeserializer());
    set.push(NewLaneDeserializer());
    set.push(AddTagDeserializer());
    set.push(AddTriggerDeserializer());
    set.push(DataSourceDeserializer());
    set.push(GetLaneDeserializer());
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
    set.push(PlotterDeserializer());
    set.push(WiggleDeserializer());
    set.push(AddAllotmentDeserializer());
    set.push(AddSwitchDeserializer());
    set.push(UseAllotmentDeserializer());
    set.push(SpaceBaseDeserializer());
    set.push(RectangleDeserializer());
    set.push(Text2Deserializer());
    set
}
