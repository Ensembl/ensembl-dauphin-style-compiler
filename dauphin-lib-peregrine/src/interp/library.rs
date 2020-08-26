use dauphin_interp::command::{ CommandSetId, CommandDeserializer, InterpLibRegister };
use dauphin_interp::runtime::{ InterpContext, Register };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::templates::NoopDeserializer;
use serde_cbor::Value as CborValue;
use super::boot::{ AddStickAuthorityDeserializer, GetStickIdDeserializer, GetStickDataDeserializer };

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xF46A437CCCFD5602)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddStickAuthorityDeserializer());
    set.push(GetStickIdDeserializer());
    set.push(GetStickDataDeserializer());
    set
}
