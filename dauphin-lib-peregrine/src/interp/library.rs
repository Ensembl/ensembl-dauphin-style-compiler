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

use super::eoes::{EoesVarNumberDeserializer, EoesVarStringDeserializer, EoesVarBooleanDeserializer, EoesNullDeserializer, EoesArrayDeserializer, EoesPairDeserializer, EoesObjectDeserializer, EoesConditionDeserializer, EoesGroupDeserializer, EoesAllDeserializer, EoesVarDeserializer, EoesNumberDeserializer, EoesStringDeserializer, EoesBooleanDeserializer, EoesLateDeserializer};
use super::track::{ 
    NewLaneDeserializer, AddTagDeserializer, AddTriggerDeserializer, DataSourceDeserializer, AddSwitchDeserializer,
    SetSwitchDeserializer, ClearSwitchDeserializer, AppendGroupDeserializer, AppendDepthDeserializer
};
use super::geometry::{
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer, PlotterDeserializer, UseAllotmentDeserializer, SpaceBaseDeserializer, SimpleColourDeserializer,
    StripedDeserializer, BarredDeserializer, BpRangeDeserializer, SpotColourDeserializer, PpcDeserializer, StyleDeserializer, PatinaSwitchDeserializer, PatinaMetadataDeserializer, BackgroundDeserializer
};
use super::shape::{
    WiggleDeserializer, RectangleDeserializer, Text2Deserializer, ImageDeserializer, EmptyDeserializer,
};

use super::switch::{
    ListSwitchDeserializer, SwitchStringDeserializer, SwitchNumberDeserializer, SwitchBooleanDeserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(52,0),0x6CECEB2729D19E37)
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
    set.push(EoesVarNumberDeserializer());
    set.push(EoesVarStringDeserializer());
    set.push(EoesVarBooleanDeserializer());
    set.push(EoesNullDeserializer());
    set.push(EoesArrayDeserializer());
    set.push(EoesPairDeserializer());
    set.push(EoesObjectDeserializer());
    set.push(EoesConditionDeserializer());
    set.push(EoesGroupDeserializer());
    set.push(EoesAllDeserializer());
    set.push(EoesVarDeserializer());
    set.push(EoesNumberDeserializer());
    set.push(EoesStringDeserializer());
    set.push(EoesBooleanDeserializer());
    set.push(EoesLateDeserializer());
    set.push(BackgroundDeserializer());
    set.push(SwitchStringDeserializer());
    set.push(SwitchNumberDeserializer());
    set.push(SwitchBooleanDeserializer());
    set
}
