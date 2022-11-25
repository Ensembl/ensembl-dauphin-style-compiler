use dauphin_interp::command::{ CommandSetId, InterpLibRegister };
use super::data::{ GetLaneDeserializer, GetDataDeserializer, OnlyWarmDeserializer, RequestScopeDeserializer, RequestDeserializer, MakeRegionDeserializer, DataNumberDeserializer, DataStringDeserializer, DataBooleanDeserializer };
use super::decompress::{ 
    SplitStringDeserializer, BaseFlipDeserializer
};

use super::eoes::{EoesVarNumberDeserializer, EoesVarStringDeserializer, EoesVarBooleanDeserializer, EoesNullDeserializer, EoesArrayDeserializer, EoesPairDeserializer, EoesObjectDeserializer, EoesConditionDeserializer, EoesGroupDeserializer, EoesAllDeserializer, EoesVarDeserializer, EoesNumberDeserializer, EoesStringDeserializer, EoesBooleanDeserializer, EoesLateDeserializer};
use super::track::{ 
    AppendGroupDeserializer, AppendDepthDeserializer
};
use super::geometry::{
    PatinaFilledDeserializer, PatinaHollowDeserializer, DirectColourDeserializer, ZMenuDeserializer, PatinaZMenuDeserializer,
    PenDeserializer, PlotterDeserializer, UseAllotmentDeserializer, SpaceBaseDeserializer, SimpleColourDeserializer,
    StripedDeserializer, BarredDeserializer, BpRangeDeserializer, PpcDeserializer, StyleDeserializer,
    PatinaMetadataDeserializer, BackgroundDeserializer, PatinaSettingSetDeserializer, PatinaSettingMemberDeserializer, PatinaSpecialZoneDeserializer
};
use super::shape::{
    WiggleDeserializer, RectangleDeserializer, Text2Deserializer, ImageDeserializer, EmptyDeserializer, RunningTextDeserializer,
};

use super::switch::{
    SettingStringDeserializer, SettingNumberDeserializer, SettingBooleanDeserializer, SettingNullDeserializer
};

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(64,0),0xA28D7DA9B2C26AB1)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(SettingStringDeserializer());
    set.push(SettingNumberDeserializer());
    set.push(SettingBooleanDeserializer());
    set.push(DataNumberDeserializer());
    set.push(DataStringDeserializer());
    set.push(DataBooleanDeserializer());
    set.push(PatinaSettingSetDeserializer());
    set.push(PatinaSettingMemberDeserializer());
    set.push(PatinaSpecialZoneDeserializer());
    set.push(SettingNullDeserializer());
    set.push(GetLaneDeserializer());
    set.push(GetDataDeserializer());
    set.push(PatinaFilledDeserializer());
    set.push(SplitStringDeserializer());
    set.push(PatinaHollowDeserializer());
    set.push(DirectColourDeserializer());
    set.push(ZMenuDeserializer());
    set.push(PatinaZMenuDeserializer());
    set.push(PenDeserializer());
    set.push(PlotterDeserializer());
    set.push(WiggleDeserializer());
    set.push(UseAllotmentDeserializer());
    set.push(SpaceBaseDeserializer());
    set.push(RectangleDeserializer());
    set.push(Text2Deserializer());
    set.push(SimpleColourDeserializer());
    set.push(StripedDeserializer());
    set.push(BarredDeserializer());
    set.push(BaseFlipDeserializer());
    set.push(OnlyWarmDeserializer());
    set.push(ImageDeserializer());
    set.push(BpRangeDeserializer());
    set.push(AppendGroupDeserializer());
    set.push(AppendDepthDeserializer());
    set.push(PpcDeserializer());
    set.push(StyleDeserializer());
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
    set.push(RunningTextDeserializer());
    set.push(MakeRegionDeserializer());
    set
}
