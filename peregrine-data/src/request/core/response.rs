use std::fmt;

use peregrine_toolkit::serde::{de_seq_next, de_wrap};
use serde::{Deserialize, Deserializer, de::{SeqAccess, Visitor}};

use crate::request::messages::{authorityres::AuthorityCommandResponse, bootstrapres::BootstrapCommandResponse, datares::DataResponse, failureres::GeneralFailure, jumpres::JumpResponse, programres::ProgramCommandResponse, stickres::StickCommandResponse};

pub struct NewCommandResponse {
    msg_id: u64,
    variety: NewResponse
}

impl NewCommandResponse {
    pub(super) fn new(msg_id: u64, variety: NewResponse) -> NewCommandResponse {
        NewCommandResponse { msg_id, variety }
    }

    pub(crate) fn message_id(&self) -> u64 { self.msg_id }
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

pub enum NewResponse {
    Bootstrap(BootstrapCommandResponse),
    GeneralFailure(GeneralFailure),
    Program(ProgramCommandResponse),
    Stick(StickCommandResponse),
    Authority(AuthorityCommandResponse),
    Data(DataResponse),
    Jump(JumpResponse)
}

impl NewResponse {
    fn bad_response(&self) -> String {
        let unexpected = match self {
            NewResponse::GeneralFailure(g) => {
                return g.message().to_string();
            },
            NewResponse::Bootstrap(_) => "bootstrap",
            NewResponse::Program(_) => "program",
            NewResponse::Stick(_) => "stick",
            NewResponse::Authority(_) => "authority",
            NewResponse::Data(_) => "data",
            NewResponse::Jump(_) => "jump"
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

    pub(crate) fn into_data(self) -> Result<DataResponse,String> {
        match self {
            NewResponse::Data(d) => Ok(d),
            _ => Err(self.bad_response())
        }
    }

    pub(crate) fn into_bootstrap(self) -> Result<BootstrapCommandResponse,String> {
        match self {
            NewResponse::Bootstrap(b) => Ok(b),
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
            0 => Ok(NewResponse::Bootstrap(de_seq_next(&mut seq)?)),
            1 => Ok(NewResponse::GeneralFailure(de_seq_next(&mut seq)?)),
            2 => Ok(NewResponse::Program(de_seq_next(&mut seq)?)),
            3 => Ok(NewResponse::Stick(de_seq_next(&mut seq)?)),
            4 => Ok(NewResponse::Authority(de_seq_next(&mut seq)?)),
            5 => Ok(NewResponse::Data(de_seq_next(&mut seq)?)),
            6 => Ok(NewResponse::Jump(de_seq_next(&mut seq)?)),
            v => {
                return de_wrap(Err(format!("bad response type: {}",v)));
            }
        }
    }
}

impl<'de> Deserialize<'de> for NewResponse {
    fn deserialize<D>(deserializer: D) -> Result<NewResponse, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(NewResponseVisitor)
    }
}
