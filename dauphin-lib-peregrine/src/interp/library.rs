use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };
use super::data::{ GetLaneDeserializer, GetDataDeserializer, DataStreamDeserializer };
use super::decompress::{ 
    InflateBytesDeserializer, InflateStringDeserializer, Lesqlite2Deserializer, ZigzagDeserializer, DeltaDeserializer,
    ClassifyDeserializer, SplitStringDeserializer
};

use super::track::{ 
    NewLaneDeserializer, AddTagDeserializer, AddTriggerDeserializer, DataSourceDeserializer, AddSwitchDeserializer,
    AddAllotmentDeserializer, SetSwitchDeserializer, ClearSwitchDeserializer
};
use super::geometry::{
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer, PlotterDeserializer, UseAllotmentDeserializer, SpaceBaseDeserializer
};
use super::shape::{
    WiggleDeserializer, RectangleDeserializer, Text2Deserializer
};

use super::switch::{
    GetSwitchDeserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(22,0),0xA21EC15B8C79F9F0)
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
    set.push(GetSwitchDeserializer());
    set.push(SetSwitchDeserializer());
    set.push(ClearSwitchDeserializer());
    set
}
