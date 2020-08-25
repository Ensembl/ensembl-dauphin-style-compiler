use dauphin_interp::command::{ CommandSetId, InterpCommand, CommandDeserializer, InterpLibRegister };
use dauphin_interp::runtime::{ InterpContext, Register };
use dauphin_interp::util::DauphinError;
use dauphin_interp::util::templates::NoopDeserializer;
use serde_cbor::Value as CborValue;
use super::boot::AddStickAuthorityDeserializer;

pub fn std_id() -> CommandSetId {
    CommandSetId::new("peregrine",(0,0),0xD6BF21A90B89A2CB)
}

pub fn make_peregrine_interp() -> InterpLibRegister {
    let mut set = InterpLibRegister::new(&std_id());
    set.push(AddStickAuthorityDeserializer());
    set
}
