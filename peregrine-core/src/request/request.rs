// TODO tied failures

use std::any::Any;
use anyhow::{ self, Context, anyhow as err };
use serde_cbor::Value as CborValue;
use std::rc::Rc;

pub trait RequestType {
    fn type_index(&self) -> u8;
    fn serialize(&self) -> anyhow::Result<CborValue>;
    fn to_failure(self) -> Box<dyn ResponseType>;
}

#[derive(Clone)]
pub struct CommandRequest(u64,Rc<Box<dyn RequestType>>);

impl CommandRequest {
    pub(crate) fn new(msgid: u64, rt: Box<dyn RequestType>) -> CommandRequest {
        CommandRequest(msgid,Rc::new(rt))
    }

    pub(crate) fn message_id(&self) -> u64 { self.0 }

    pub fn serialize(&self) -> anyhow::Result<CborValue> {
        let typ = self.1.type_index();
        Ok(CborValue::Array(vec![CborValue::Integer(self.0 as i128),CborValue::Integer(typ as i128),self.1.serialize()?]))
    }
}

pub struct CommandResponse(u64,Box<dyn ResponseType>);

impl CommandResponse {
    pub(crate) fn new(msgid: u64, rt: Box<dyn ResponseType>) -> CommandResponse {
        CommandResponse(msgid,rt)
    }

    pub(crate) fn message_id(&self) -> u64 { self.0 }
    pub(crate) fn into_response(self) -> Box<dyn ResponseType> { self.1 }
}

pub trait ResponseType {
    fn as_any(&self) -> &Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait ResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>>;
}
