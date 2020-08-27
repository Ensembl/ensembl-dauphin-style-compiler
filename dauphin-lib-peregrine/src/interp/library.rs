use dauphin_interp::command::{ CommandSetId, CommandDeserializer, InterpLibRegister };
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer, AddStickDeserializer };

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xE4F0C0276A75C1A9)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddStickAuthorityDeserializer());
    set.push(GetStickIdDeserializer());
    set.push(GetStickDataDeserializer());
    set.push(AddStickDeserializer());
    set
}
