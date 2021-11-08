// TODO tied failures
use crate::request::messages::authorityreq::AuthorityCommandRequest;
use crate::request::messages::bootstrapreq::BootstrapCommandRequest;
use crate::request::messages::datareq::DataCommandRequest;
use crate::request::messages::failureres::GeneralFailure;
use crate::request::messages::jumpreq::JumpCommandRequest;
use crate::request::messages::programreq::ProgramCommandRequest;
use crate::request::messages::stickreq::StickCommandRequest;
use crate::{metric::metricreporter::MetricReport};
use std::sync::Arc;
use serde::{Serializer, ser::SerializeSeq};
use std::rc::Rc;
use super::response::NewCommandResponse;
use super::response::NewResponse;

#[derive(Clone)]
pub struct RequestType {
    variant: Arc<NewRequestVariant>
}

impl RequestType {
    pub(crate) fn new(variant: NewRequestVariant) -> RequestType {
        RequestType {
            variant: Arc::new(variant)
        }
    }
}

pub(crate) enum NewRequestVariant {
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
            NewRequestVariant::Bootstrap(_) => "bootstrapping",
            NewRequestVariant::Program(_) => "getting program",
            NewRequestVariant::Stick(_) => "getting stick info",
            NewRequestVariant::Authority(_) => "getting authority info",
            NewRequestVariant::Data(_) => "getting data",
            NewRequestVariant::Jump(_) => "getting jump location",
            NewRequestVariant::Metric(_) => "sending metric report",
        };
        NewResponse::GeneralFailure(GeneralFailure::new(out))
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

    pub(crate) fn to_failure(&self) -> NewResponse { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> NewCommandResponse {
        NewCommandResponse::new(self.msgid,self.data.to_failure())
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
