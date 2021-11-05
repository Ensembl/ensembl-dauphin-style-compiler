// TODO tied failures

use crate::{metric::metricreporter::MetricReport };
use std::{any::Any, sync::Arc};
use anyhow::{ self };
use serde::{Serializer, ser::SerializeSeq};
use serde_cbor::Value as CborValue;
use std::rc::Rc;

use super::{authority::AuthorityCommandRequest, bootstrap::BootstrapCommandRequest, data::DataCommandRequest, failure::GeneralFailure, jump::JumpCommandRequest, program::ProgramCommandRequest, stick::StickCommandRequest};

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
    pub fn to_failure(&self) -> Box<dyn ResponseType> {
        match self.variant.as_ref() {
            NewRequestVariant::Bootstrap(_) => Box::new(GeneralFailure::new("bootstrapping")),
            NewRequestVariant::Program(_) => Box::new(GeneralFailure::new("getting program")),
            NewRequestVariant::Stick(_) => Box::new(GeneralFailure::new("getting stick info")),
            NewRequestVariant::Authority(_) => Box::new(GeneralFailure::new("getting authority info")),
            NewRequestVariant::Data(_) => Box::new(GeneralFailure::new("getting data")),
            NewRequestVariant::Jump(_) => Box::new(GeneralFailure::new("getting jump location")),
            NewRequestVariant::Metric(_) => Box::new(GeneralFailure::new("sending metric report")),
        }
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

    pub(super) fn to_failure(&self) -> Box<dyn ResponseType> { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> CommandResponse {
        CommandResponse::new(self.msgid,self.data.to_failure())
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
