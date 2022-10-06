// TODO tied failures
use crate::request::minirequests::authorityreq::AuthorityReq;
use crate::request::minirequests::bootchannelreq::BootChannelReq;
use crate::request::minirequests::datareq::DataRequest;
use crate::request::minirequests::failureres::FailureRes;
use crate::request::minirequests::jumpreq::JumpReq;
use crate::request::minirequests::metricreq::MetricReport;
use crate::request::minirequests::programreq::ProgramReq;
use crate::request::minirequests::stickreq::StickReq;
use std::rc::Rc;
use super::response::BackendResponseAttempt;
use super::response::BackendResponse;
use commander::CommanderStream;

pub trait MiniRequestVariety {
    fn description(&self) -> String;
}

pub enum MiniRequest {
    BootChannel(BootChannelReq),
    Program(ProgramReq),
    Stick(StickReq),
    Authority(AuthorityReq),
    Data(DataRequest),
    Jump(JumpReq),
    Metric(MetricReport)
}

impl MiniRequest {
    fn as_mini(&self) -> &dyn MiniRequestVariety {
        match self {
            MiniRequest::BootChannel(x) => x,
            MiniRequest::Program(x) => x,
            MiniRequest::Stick(x) => x,
            MiniRequest::Authority(x) => x,
            MiniRequest::Data(x) => x,
            MiniRequest::Jump(x) => x,
            MiniRequest::Metric(x) => x
        }
    }
}

#[derive(Clone)]
pub struct MiniRequestAttempt {
    msgid: u64,
    description: String,
    request: Rc<MiniRequest>,
    response: CommanderStream<BackendResponse>
}

impl MiniRequestAttempt {
    pub(crate) fn new(msgid: u64, request: &Rc<MiniRequest>) -> MiniRequestAttempt {
        MiniRequestAttempt {
            msgid,
            request: request.clone(),
            description: request.as_mini().description(),
            response: CommanderStream::new()
        }
    }

    pub(crate) fn response(&self) -> &CommanderStream<BackendResponse> { &self.response }

    pub(crate) fn fail(&self) -> BackendResponseAttempt {
        let failure = BackendResponse::FailureRes(FailureRes::new(&self.description));
        BackendResponseAttempt::new(self.msgid,failure)
    }

    pub fn msgid(&self) -> u64 { self.msgid }
    pub fn request(&self) -> &MiniRequest { &self.request }
}
