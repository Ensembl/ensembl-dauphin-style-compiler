// TODO tied failures
use crate::request::messages::authorityreq::AuthorityCommandRequest;
use crate::request::messages::bootstrapreq::BootstrapCommandRequest;
use crate::request::messages::datareq::DataCommandRequest;
use crate::request::messages::failureres::GeneralFailure;
use crate::request::messages::jumpreq::JumpCommandRequest;
use crate::request::messages::metricreq::MetricReport;
use crate::request::messages::programreq::ProgramCommandRequest;
use crate::request::messages::stickreq::StickCommandRequest;
use std::sync::Arc;
use std::rc::Rc;
use super::response::NewCommandResponse;
use super::response::NewResponse;
use serde_cbor::Value as CborValue;

pub(crate) enum RequestVariant {
    Bootstrap(BootstrapCommandRequest),
    Program(ProgramCommandRequest),
    Stick(StickCommandRequest),
    Authority(AuthorityCommandRequest),
    Data(DataCommandRequest),
    Jump(JumpCommandRequest),
    Metric(MetricReport),
}

#[derive(Clone)]
pub struct RequestType {
    variant: Arc<RequestVariant>
}

impl RequestType {
    pub(crate) fn new(variant: RequestVariant) -> RequestType {
        RequestType {
            variant: Arc::new(variant)
        }
    }

    fn type_index(&self) -> u8 {
        match self.variant.as_ref() {
            RequestVariant::Bootstrap(_) => 0,
            RequestVariant::Program(_) => 1,
            RequestVariant::Stick(_) => 2,
            RequestVariant::Authority(_) => 3,
            RequestVariant::Data(_) => 4,
            RequestVariant::Jump(_) => 5,
            RequestVariant::Metric(_) => 6,
        }
    }

    pub fn to_failure(&self) -> NewResponse {
        let out = match self.variant.as_ref() {
            RequestVariant::Bootstrap(_) => "bootstrap",
            RequestVariant::Program(_) => "program",
            RequestVariant::Stick(_) => "stick",
            RequestVariant::Authority(_) => "authority",
            RequestVariant::Data(_) => "data",
            RequestVariant::Jump(_) => "jump",
            RequestVariant::Metric(_) => "metric",

        };
        NewResponse::GeneralFailure(GeneralFailure::new(out))
    }

    pub(crate) fn encode(&self) -> CborValue {
        match self.variant.as_ref() {
            RequestVariant::Bootstrap(x) => x.encode(),
            RequestVariant::Program(x) => x.encode(),
            RequestVariant::Stick(x) => x.encode(),
            RequestVariant::Authority(x) => x.encode(),
            RequestVariant::Data(x) => x.encode(),
            RequestVariant::Jump(x) => x.encode(),
            RequestVariant::Metric(x) => x.encode(),
        }
    }
}

#[derive(Clone)]
pub struct CommandRequest {
    msgid: u64,
    data: Rc<RequestType>
}

impl CommandRequest {
    pub(crate) fn new2(msgid: u64, rt: RequestType) -> CommandRequest {
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

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Integer(self.msgid as i128),
            CborValue::Integer(self.data.type_index() as i128),
            self.data.encode()
        ])
    }
}
