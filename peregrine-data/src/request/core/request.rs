// TODO tied failures
use crate::request::minirequests::bootchannelreq::BootChannelReq;
use crate::request::minirequests::datareq::DataRequest;
use crate::request::minirequests::expandreq::ExpandReq;
use crate::request::minirequests::failureres::FailureRes;
use crate::request::minirequests::jumpreq::JumpReq;
use crate::request::minirequests::metricreq::MetricReport;
use crate::request::minirequests::programreq::ProgramReq;
use crate::request::minirequests::stickreq::StickReq;
use std::rc::Rc;
use super::response::MiniResponseAttempt;
use super::response::MiniResponse;
use commander::CommanderStream;
use serde::{ Serialize };
use serde::ser::SerializeSeq;

pub trait MiniRequestVariety {
    fn description(&self) -> String;
    fn opcode(&self) -> u8;
}

pub enum MiniRequest {
    BootChannel(BootChannelReq),
    Program(ProgramReq),
    Stick(StickReq),
    Data(DataRequest),
    Jump(JumpReq),
    Metric(MetricReport),
    Expand(ExpandReq)
}

impl MiniRequest {
    pub fn as_mini(&self) -> &dyn MiniRequestVariety {
        match self {
            MiniRequest::BootChannel(x) => x,
            MiniRequest::Program(x) => x,
            MiniRequest::Stick(x) => x,
            MiniRequest::Data(x) => x,
            MiniRequest::Jump(x) => x,
            MiniRequest::Metric(x) => x,
            MiniRequest::Expand(x) => x
        }
    }
}

impl Serialize for MiniRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        match self {
            MiniRequest::BootChannel(x) => x.serialize(serializer),
            MiniRequest::Program(x) =>  x.serialize(serializer),
            MiniRequest::Stick(x) =>  x.serialize(serializer),
            MiniRequest::Data(x) =>  x.serialize(serializer),
            MiniRequest::Jump(x) =>  x.serialize(serializer),
            MiniRequest::Metric(x) => x.serialize(serializer),
            MiniRequest::Expand(x) => x.serialize(serializer),
        }
    }
}

#[derive(Clone)]
pub struct MiniRequestAttempt {
    msgid: u64,
    description: String,
    request: Rc<MiniRequest>,
    response: CommanderStream<MiniResponseAttempt>
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

    pub(crate) fn response(&self) -> &CommanderStream<MiniResponseAttempt> { &self.response }

    pub fn fail(&self, extra: &str) -> MiniResponseAttempt {
        let message = format!("{}: {}",self.description,extra);
        let failure = MiniResponse::FailureRes(FailureRes::new(&message));
        MiniResponseAttempt::new(self.msgid,failure)
    }

    pub fn make_response_attempt(&self, mini: MiniResponse) -> MiniResponseAttempt {
        MiniResponseAttempt::new(self.msgid,mini)
    }

    pub fn msgid(&self) -> u64 { self.msgid }
    pub fn request(&self) -> &MiniRequest { &self.request }
}

impl Serialize for MiniRequestAttempt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.msgid)?;
        seq.serialize_element(&self.request.as_mini().opcode())?;
        seq.serialize_element(&self.request)?;
        seq.end()
    }
}
