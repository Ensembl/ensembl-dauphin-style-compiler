use dauphin_interp::command::{ CommandSetId, CommandDeserializer, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };
use super::panel::{ NewPanelDeserializer, AddTagDeserializer, AddTrackDeserializer, SetScaleDeserializer, DataSourceDeserializer };

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0x748A24A0A2D68971)
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
    set
}
