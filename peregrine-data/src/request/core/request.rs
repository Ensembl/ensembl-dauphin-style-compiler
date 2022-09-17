// TODO tied failures
use crate::request::messages::authorityreq::AuthorityReq;
use crate::request::messages::bootstrapreq::BootstrapReq;
use crate::request::messages::datareq::DataRequest;
use crate::request::messages::failureres::FailureRes;
use crate::request::messages::jumpreq::JumpReq;
use crate::request::messages::metricreq::MetricReport;
use crate::request::messages::programreq::ProgramReq;
use crate::request::messages::stickreq::StickReq;
use std::rc::Rc;
use super::response::BackendResponseAttempt;
use super::response::BackendResponse;
use serde_cbor::Value as CborValue;

pub struct DataRequestSerialization {
    failure_string: String,
    encoding: CborValue    
}

impl DataRequestSerialization {
    fn make_type_index(request: &BackendRequest) -> u8 {
        match request {
            BackendRequest::Bootstrap(_) => 0,
            BackendRequest::Program(_) => 1,
            BackendRequest::Stick(_) => 2,
            BackendRequest::Authority(_) => 3,
            BackendRequest::Data(_) => 4,
            BackendRequest::Jump(_) => 5,
            BackendRequest::Metric(_) => 6,
        }
    }

    fn make_to_failure(request: &BackendRequest) -> String {
        match request {
            BackendRequest::Bootstrap(_) => "bootstrap",
            BackendRequest::Program(_) => "program",
            BackendRequest::Stick(_) => "stick",
            BackendRequest::Authority(_) => "authority",
            BackendRequest::Data(_) => "data",
            BackendRequest::Jump(_) => "jump",
            BackendRequest::Metric(_) => "metric",
        }.to_string()
    }

    fn make_encode(request: &BackendRequest, msgid: u64) -> CborValue {
        CborValue::Array(vec![
            CborValue::Integer(msgid as i128),
            CborValue::Integer(Self::make_type_index(request) as i128),
            Self::make_encode_data(request)
        ])
    }


    fn make_encode_data(request: &BackendRequest) -> CborValue {
        match request {
            BackendRequest::Bootstrap(x) => x.encode(),
            BackendRequest::Program(x) => x.encode(),
            BackendRequest::Stick(x) => x.encode(),
            BackendRequest::Authority(x) => x.encode(),
            BackendRequest::Data(x) => x.encode(),
            BackendRequest::Jump(x) => x.encode(),
            BackendRequest::Metric(x) => x.encode(),
        }
    }

    fn new(request: &BackendRequest, msgid: u64) -> DataRequestSerialization {
        DataRequestSerialization {
             failure_string: Self::make_to_failure(request),
             encoding: Self::make_encode(request,msgid)
        }
    }

    pub fn to_failure(&self) -> BackendResponse {
        BackendResponse::FailureRes(FailureRes::new(&self.failure_string))
    }

    pub(crate) fn encode(&self) -> &CborValue {
        &self.encoding
    }
}

pub(crate) enum BackendRequest {
    Bootstrap(BootstrapReq),
    Program(ProgramReq),
    Stick(StickReq),
    Authority(AuthorityReq),
    Data(DataRequest),
    Jump(JumpReq),
    Metric(MetricReport),
}

#[derive(Clone)]
pub struct BackendRequestAttempt {
    msgid: u64,
    data: Rc<DataRequestSerialization>
}

impl BackendRequestAttempt {
    pub(crate) fn new(msgid: u64, data: &BackendRequest) -> BackendRequestAttempt {
        BackendRequestAttempt {
            msgid,
            data: Rc::new(DataRequestSerialization::new(data,msgid))
        }
    }

    pub(crate) fn to_failure(&self) -> BackendResponse { self.data.to_failure() }
    pub(crate) fn message_id(&self) -> u64 { self.msgid }
    pub(crate) fn fail(&self) -> BackendResponseAttempt {
        BackendResponseAttempt::new(self.msgid,self.data.to_failure())
    }

    pub(crate) fn encode(&self) -> &CborValue { self.data.encode() }
}
