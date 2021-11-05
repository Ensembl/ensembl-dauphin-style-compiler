// TODO tied failures

use crate::{util::serde::ser_wrap};
use std::{any::Any, sync::Arc};
use anyhow::{ self };
use serde::{Serializer, ser::SerializeSeq};
use serde_cbor::Value as CborValue;
use std::rc::Rc;
use crate::util::message::DataMessage;

use super::{authority::AuthorityCommandRequest, failure::GeneralFailure, jump::JumpCommandRequest, program::ProgramCommandRequest, stick::StickCommandRequest};

pub trait OldRequestType {
    fn type_index(&self) -> u8;
    fn serialize(&self) -> Result<CborValue,DataMessage>;
    fn to_failure(&self) -> Box<dyn ResponseType>;
}

#[derive(Clone)]
pub struct NewRequestType {
    variant: Arc<NewRequestVariant>
}

impl NewRequestType {
    pub(super) fn new_jump(request: JumpCommandRequest) -> NewRequestType {
        NewRequestType {
            variant: Arc::new(NewRequestVariant::Jump(request))
        }
    }

    pub(super) fn new_program(request: ProgramCommandRequest) -> NewRequestType {
        NewRequestType {
            variant: Arc::new(NewRequestVariant::Program(request))
        }
    }

    pub(super) fn new_stick(request: StickCommandRequest) -> NewRequestType {
        NewRequestType {
            variant: Arc::new(NewRequestVariant::Stick(request))
        }
    }

    pub(super) fn new_authority(request: AuthorityCommandRequest) -> NewRequestType {
        NewRequestType {
            variant: Arc::new(NewRequestVariant::Authority(request))
        }
    }
}

enum NewRequestVariant {
    Program(ProgramCommandRequest),
    Stick(StickCommandRequest),
    Authority(AuthorityCommandRequest),
    Jump(JumpCommandRequest)
}

impl serde::Serialize for NewRequestType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self.variant.as_ref() {
            NewRequestVariant::Jump(x) => x.serialize(serializer),
            NewRequestVariant::Program(x) => x.serialize(serializer),
            NewRequestVariant::Stick(x) => x.serialize(serializer),
            NewRequestVariant::Authority(x) => x.serialize(serializer)
        }
    }
}

impl NewRequestType {
    pub fn to_failure(&self) -> Box<dyn ResponseType> {
        match self.variant.as_ref() {
            NewRequestVariant::Jump(_) => Box::new(GeneralFailure::new("getting jump location")),
            NewRequestVariant::Program(_) => Box::new(GeneralFailure::new("getting program")),
            NewRequestVariant::Stick(_) => Box::new(GeneralFailure::new("getting stick info")),
            NewRequestVariant::Authority(_) => Box::new(GeneralFailure::new("getting authority info")),
        }
    }

    fn type_index(&self) -> u8 {
        match self.variant.as_ref() {
            NewRequestVariant::Program(_) => 1,
            NewRequestVariant::Stick(_) => 2,
            NewRequestVariant::Authority(_) => 3,
            NewRequestVariant::Jump(_) => 5,
        }
    }
}

#[derive(Clone)]
pub struct OldCommandRequest {
    msgid: u64,
    data: Rc<Box<dyn OldRequestType>>
}

#[derive(Clone)]
pub struct NewCommandRequest {
    msgid: u64,
    data: Rc<NewRequestType>
}

#[derive(Clone)]
pub enum CommandRequest {
    Old(OldCommandRequest),
    New(NewCommandRequest)
}

impl OldCommandRequest {
    pub(crate) fn new(msgid: u64, rt: Box<dyn OldRequestType>) -> OldCommandRequest {
        OldCommandRequest {
            msgid,
            data: Rc::new(rt)
        }
    }

    pub(super) fn to_failure(&self) -> Box<dyn ResponseType> { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> CommandResponse {
        CommandResponse::new(self.msgid,self.data.to_failure())
    }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        let typ = self.data.type_index();
        Ok(CborValue::Array(vec![CborValue::Integer(self.msgid as i128),CborValue::Integer(typ as i128),self.data.serialize()?]))
    }
}

impl NewCommandRequest {
    pub(crate) fn new(msgid: u64, rt: NewRequestType) -> NewCommandRequest {
        NewCommandRequest {
            msgid,
            data: Rc::new(rt)
        }
    }

    pub(super) fn to_failure(&self) -> Box<dyn ResponseType> { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> CommandResponse {
        CommandResponse::new(self.msgid,self.data.to_failure())
    }
}

impl serde::Serialize for NewCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.msgid)?;
        seq.serialize_element(&self.data.type_index())?;
        seq.serialize_element(&self.data)?;
        seq.end()
    }
}

impl CommandRequest {
    pub(crate) fn message_id(&self) -> u64 {
        match self {
            CommandRequest::New(x) => x.message_id(),
            CommandRequest::Old(x) => x.message_id()
        }
    }

    pub(crate) fn fail(&self) -> CommandResponse {
        match self {
            CommandRequest::New(x) => x.fail(),
            CommandRequest::Old(x) => x.fail()
        }
    }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        match self {
            CommandRequest::New(x) => {
                let xxx_value = ser_wrap(serde_cbor::to_vec(&x))?;
                ser_wrap(serde_cbor::from_slice(&xxx_value))      
            },
            CommandRequest::Old(x) => x.serialize()
        }
    }

    pub(super) fn to_failure(&self) -> Box<dyn ResponseType> {
        match self {
            CommandRequest::New(x) => x.to_failure(),
            CommandRequest::Old(x) => x.to_failure()
        }
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
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait ResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>>;
}
