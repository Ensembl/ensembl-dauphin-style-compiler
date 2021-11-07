// TODO tied failures
use anyhow::anyhow as err;
use anyhow::Context;
use peregrine_toolkit::serde::{ de_seq_next, de_wrap };
use serde::Deserialize;
use serde::Deserializer;
use serde::de::SeqAccess;
use serde::de::Visitor;
use crate::request::queue::register_responses;
use crate::{metric::metricreporter::MetricReport};
use std::fmt;
use std::{any::Any, sync::Arc};
use anyhow::{ self };
use serde::{Serializer, ser::SerializeSeq};
use serde_cbor::Value as CborValue;
use std::rc::Rc;

use super::authority::AuthorityCommandResponse;
use super::program::ProgramCommandResponse;
use super::stick::StickCommandResponse;
use super::{authority::AuthorityCommandRequest, bootstrap::BootstrapCommandRequest, data::DataCommandRequest, failure::GeneralFailure, jump::{JumpCommandRequest, JumpResponse}, program::ProgramCommandRequest, stick::StickCommandRequest};

#[derive(Clone)]
pub struct RequestType {
    variant: Arc<NewRequestVariant>
}

impl RequestType {
    pub(super) fn new_bootstrap(request: BootstrapCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Bootstrap(request))
        }
    }

    pub(super) fn new_jump(request: JumpCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Jump(request))
        }
    }

    pub(super) fn new_program(request: ProgramCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Program(request))
        }
    }

    pub(super) fn new_stick(request: StickCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Stick(request))
        }
    }

    pub(super) fn new_authority(request: AuthorityCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Authority(request))
        }
    }

    pub(super) fn new_data(request: DataCommandRequest) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Data(request))
        }
    }

    pub(crate) fn new_metric(request: MetricReport) -> RequestType {
        RequestType {
            variant: Arc::new(NewRequestVariant::Metric(request))
        }
    }
}

enum NewRequestVariant {
    Bootstrap(BootstrapCommandRequest),
    Program(ProgramCommandRequest),
    Stick(StickCommandRequest),
    Authority(AuthorityCommandRequest),
    Data(DataCommandRequest),
    Jump(JumpCommandRequest),
    Metric(MetricReport),
}

impl serde::Serialize for RequestType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self.variant.as_ref() {
            NewRequestVariant::Bootstrap(x) => x.serialize(serializer),
            NewRequestVariant::Program(x) => x.serialize(serializer),
            NewRequestVariant::Stick(x) => x.serialize(serializer),
            NewRequestVariant::Authority(x) => x.serialize(serializer),
            NewRequestVariant::Data(x) => x.serialize(serializer),
            NewRequestVariant::Jump(x) => x.serialize(serializer),
            NewRequestVariant::Metric(x) => x.serialize(serializer),
        }
    }
}

impl RequestType {
    pub fn to_failure(&self) -> NewResponse {
        let out = match self.variant.as_ref() {
            NewRequestVariant::Bootstrap(_) => GeneralFailure::new("bootstrapping"),
            NewRequestVariant::Program(_) => GeneralFailure::new("getting program"),
            NewRequestVariant::Stick(_) => GeneralFailure::new("getting stick info"),
            NewRequestVariant::Authority(_) => GeneralFailure::new("getting authority info"),
            NewRequestVariant::Data(_) => GeneralFailure::new("getting data"),
            NewRequestVariant::Jump(_) => GeneralFailure::new("getting jump location"),
            NewRequestVariant::Metric(_) => GeneralFailure::new("sending metric report"),
        };
        NewResponse::GeneralFailure(out)
    }

    fn type_index(&self) -> u8 {
        match self.variant.as_ref() {
            NewRequestVariant::Bootstrap(_) => 0,
            NewRequestVariant::Program(_) => 1,
            NewRequestVariant::Stick(_) => 2,
            NewRequestVariant::Authority(_) => 3,
            NewRequestVariant::Data(_) => 4,
            NewRequestVariant::Jump(_) => 5,
            NewRequestVariant::Metric(_) => 6,
        }
    }
}

#[derive(Clone)]
pub struct CommandRequest {
    msgid: u64,
    data: Rc<RequestType>
}

impl CommandRequest {
    pub(crate) fn new(msgid: u64, rt: RequestType) -> CommandRequest {
        CommandRequest {
            msgid,
            data: Rc::new(rt)
        }
    }

    pub(super) fn to_failure(&self) -> NewResponse { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> NewCommandResponse {
        NewCommandResponse {
            msg_id: self.msgid,
            variety: self.data.to_failure()
        }
    }
}

impl serde::Serialize for CommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.msgid)?;
        seq.serialize_element(&self.data.type_index())?;
        seq.serialize_element(&self.data)?;
        seq.end()
    }
}

pub struct NewCommandResponse {
    msg_id: u64,
    variety: NewResponse
}

impl NewCommandResponse {
    pub(super) fn message_id(&self) -> u64 { self.msg_id }
    pub(crate) fn into_variety(self) -> NewResponse { self.variety }
}

struct CommandResponseVisitor;

impl<'de> Visitor<'de> for CommandResponseVisitor {
    type Value = NewCommandResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a command response") }

    fn visit_seq<S>(self, mut seq: S) -> Result<NewCommandResponse,S::Error> where S: SeqAccess<'de> {
        let msg_id = de_seq_next(&mut seq)?;
        let variety = de_seq_next(&mut seq)?;
        Ok(NewCommandResponse { msg_id, variety })
    }
}

impl<'de> Deserialize<'de> for NewCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<NewCommandResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(CommandResponseVisitor)
    }
}

pub trait ResponseType {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait ResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>>;
}

pub enum NewResponse {
    GeneralFailure(GeneralFailure),
    Program(ProgramCommandResponse),
    Stick(StickCommandResponse),
    Authority(AuthorityCommandResponse),
    Jump(JumpResponse),
    Other(Box<dyn ResponseType>)
}

impl NewResponse {
    fn bad_response(&self) -> String {
        let unexpected = match self {
            NewResponse::GeneralFailure(g) => {
                return g.message().to_string();
            },
            NewResponse::Program(_) => "program",
            NewResponse::Stick(_) => "stick",
            NewResponse::Authority(_) => "authority",
            NewResponse::Jump(_) => "jump",
            NewResponse::Other(_) => "unknown"
        };
        format!("unexpected response: {}",unexpected)
    }

    pub(crate) fn into_jump(self) -> Result<JumpResponse,String> {
        match self {
            NewResponse::Jump(j) => Ok(j),
            _ => Err(self.bad_response())
        }
    }

    pub(crate) fn into_program(self) -> Result<ProgramCommandResponse,String> {
        match self {
            NewResponse::Program(p) => Ok(p),
            _ => Err(self.bad_response())
        }
    }

    pub(crate) fn into_stick(self) -> Result<StickCommandResponse,String> {
        match self {
            NewResponse::Stick(s) => Ok(s),
            _ => Err(self.bad_response())
        }
    }

    pub(crate) fn into_authority(self) -> Result<AuthorityCommandResponse,String> {
        match self {
            NewResponse::Authority(a) => Ok(a),
            _ => Err(self.bad_response())
        }
    }
}

struct NewResponseVisitor;

impl<'de> Visitor<'de> for NewResponseVisitor {
    type Value = NewResponse;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a response") }

    fn visit_seq<S>(self, mut seq: S) -> Result<NewResponse,S::Error> where S: SeqAccess<'de> {
        let variety = de_seq_next(&mut seq)?;
        match variety {
            1 => Ok(NewResponse::GeneralFailure(de_seq_next(&mut seq)?)),
            2 => Ok(NewResponse::Program(de_seq_next(&mut seq)?)),
            3 => Ok(NewResponse::Stick(de_seq_next(&mut seq)?)),
            4 => Ok(NewResponse::Authority(de_seq_next(&mut seq)?)),
            6 => Ok(NewResponse::Jump(de_seq_next(&mut seq)?)),
            _ => {
                let payload : CborValue = de_seq_next(&mut seq)?;
                let buffer = de_wrap(serde_cbor::to_vec(&payload))?;
                let data = de_wrap(serde_cbor::from_slice(&buffer))?;
                let builders = register_responses().builders;                                                                                                                                                                            
                let builder = de_wrap(builders.get(&variety).ok_or(err!("bad response type")))?;
                let payload = de_wrap(builder.deserialize(&data).with_context(
                    || format!("deserializing individual response payload (type {})",variety)))?;
                Ok(NewResponse::Other(payload))
            }
        }
    }
}

impl<'de> Deserialize<'de> for NewResponse {
    fn deserialize<D>(deserializer: D) -> Result<NewResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(NewResponseVisitor)
    }
}
