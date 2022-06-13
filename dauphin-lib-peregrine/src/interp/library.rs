use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::boot::{ 
    AddAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer,
    AddJumpDeserializer, GetJumpDataDeserializer, GetJumpLocationDeserializer
};
use super::data::{ GetLaneDeserializer, GetDataDeserializer, DataStreamDeserializer, OnlyWarmDeserializer, RequestScopeDeserializer, RequestDeserializer };
use super::decompress::{ 
    InflateBytesDeserializer, InflateStringDeserializer, Lesqlite2Deserializer, ZigzagDeserializer, DeltaDeserializer,
    ClassifyDeserializer, SplitStringDeserializer, BaseFlipDeserializer
};

use super::track::{ 
    NewLaneDeserializer, AddTagDeserializer, AddTriggerDeserializer, DataSourceDeserializer, AddSwitchDeserializer,
    SetSwitchDeserializer, ClearSwitchDeserializer, AppendGroupDeserializer, AppendDepthDeserializer
};
use super::geometry::{
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer, PlotterDeserializer, UseAllotmentDeserializer, SpaceBaseDeserializer, SimpleColourDeserializer,
    StripedDeserializer, BarredDeserializer, BpRangeDeserializer, SpotColourDeserializer, PpcDeserializer, StyleDeserializer, PatinaSwitchDeserializer, PatinaMetadataDeserializer
};
use super::shape::{
    WiggleDeserializer, RectangleDeserializer, Text2Deserializer, ImageDeserializer, EmptyDeserializer,
};

use super::switch::{
    GetSwitchDeserializer, ListSwitchDeserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(51,0),0xA6BD2CCD09C72076)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddAuthorityDeserializer());
    set.push(GetStickIdDeserializer());
    set.push(GetJumpLocationDeserializer());
    set.push(GetStickDataDeserializer());
    set.push(GetJumpDataDeserializer());
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
    set.push(AddSwitchDeserializer());
    set.push(UseAllotmentDeserializer());
    set.push(SpaceBaseDeserializer());
    set.push(RectangleDeserializer());
    set.push(Text2Deserializer());
    set.push(GetSwitchDeserializer());
    set.push(SetSwitchDeserializer());
    set.push(ClearSwitchDeserializer());
    set.push(SimpleColourDeserializer());
    set.push(StripedDeserializer());
    set.push(BarredDeserializer());
    set.push(BaseFlipDeserializer());
    set.push(AddJumpDeserializer());
    set.push(ListSwitchDeserializer());
    set.push(OnlyWarmDeserializer());
    set.push(ImageDeserializer());
    set.push(BpRangeDeserializer());
    set.push(SpotColourDeserializer());
    set.push(AppendGroupDeserializer());
    set.push(AppendDepthDeserializer());
    set.push(PpcDeserializer());
    set.push(StyleDeserializer());
    set.push(PatinaSwitchDeserializer());
    set.push(RequestDeserializer());
    set.push(RequestScopeDeserializer());
    set.push(EmptyDeserializer());
    set.push(PatinaMetadataDeserializer());
    set
}
