// TODO tied failures
use crate::request::messages::authorityreq::AuthorityReq;
use crate::request::messages::bootstrapreq::BootstrapReq;
use crate::request::messages::datareq::DataRequest;
use crate::request::messages::failureres::FailureRes;
use crate::request::messages::jumpreq::JumpReq;
use crate::request::messages::metricreq::MetricReport;
use crate::request::messages::programreq::ProgramReq;
use crate::request::messages::stickreq::StickReq;
use std::sync::Arc;
use std::rc::Rc;
use super::response::BackendResponseAttempt;
use super::response::BackendResponse;
use serde_cbor::Value as CborValue;

pub(crate) enum RequestVariant {
    Bootstrap(BootstrapReq),
    Program(ProgramReq),
    Stick(StickReq),
    Authority(AuthorityReq),
    Data(DataRequest),
    Jump(JumpReq),
    Metric(MetricReport),
}

#[derive(Clone)]
pub struct BackendRequest {
    variant: Arc<RequestVariant>
}

impl BackendRequest {
    pub(crate) fn new(variant: RequestVariant) -> BackendRequest {
        BackendRequest {
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

    pub fn to_failure(&self) -> BackendResponse {
        let out = match self.variant.as_ref() {
            RequestVariant::Bootstrap(_) => "bootstrap",
            RequestVariant::Program(_) => "program",
            RequestVariant::Stick(_) => "stick",
            RequestVariant::Authority(_) => "authority",
            RequestVariant::Data(_) => "data",
            RequestVariant::Jump(_) => "jump",
            RequestVariant::Metric(_) => "metric",

        };
        BackendResponse::FailureRes(FailureRes::new(out))
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
pub struct BackendRequestAttempt {
    msgid: u64,
    data: Rc<BackendRequest>
}

impl BackendRequestAttempt {
    pub(crate) fn new2(msgid: u64, rt: BackendRequest) -> BackendRequestAttempt {
        BackendRequestAttempt {
            msgid,
            data: Rc::new(rt)
        }
    }

    pub(crate) fn to_failure(&self) -> BackendResponse { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> BackendResponseAttempt {
        BackendResponseAttempt::new(self.msgid,self.data.to_failure())
    }

    pub(crate) fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            CborValue::Integer(self.msgid as i128),
            CborValue::Integer(self.data.type_index() as i128),
            self.data.encode()
        ])
    }
}
