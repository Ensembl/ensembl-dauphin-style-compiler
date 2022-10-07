use std::fmt;
use peregrine_toolkit::{serdetools::st_field};
use serde::{Deserialize, Deserializer, de::Visitor};
use crate::request::minirequests::{authorityres::AuthorityRes, bootchannelres::BootChannelRes, datares::DataRes, failureres::FailureRes, jumpres::JumpRes, programres::ProgramRes, stickres::StickRes };

pub(crate) trait MiniResponseVariety {
    fn description(&self) -> &str;
    fn total_size(&self) -> usize { 0 }
    fn component_size(&self) -> Vec<(String,usize)> { vec![] }
}

pub enum MiniResponse {
    BootChannel(BootChannelRes),
    FailureRes(FailureRes),
    Program(ProgramRes),
    Stick(StickRes),
    Authority(AuthorityRes),
    Data(DataRes),
    Jump(JumpRes)
}

macro_rules! accessor {
    ($self:ident,$name:tt,$branch:tt,$result:ty) => {
        pub(crate) fn $name($self) -> Result<$result,String> {
            match $self {
                MiniResponse::$branch(j) => Ok(j),
                _ => Err($self.bad_response())
            }
        }
    };
}

impl MiniResponse {
    fn as_mini(&self) -> &dyn MiniResponseVariety {
        match self {
            MiniResponse::BootChannel(x) => x,
            MiniResponse::FailureRes(x) => x,
            MiniResponse::Program(x) => x,
            MiniResponse::Stick(x) => x,
            MiniResponse::Authority(x) => x,
            MiniResponse::Data(x) => x,
            MiniResponse::Jump(x) => x
        }
    }

    fn bad_response(&self) -> String {
        match self {
            MiniResponse::FailureRes(g) => { g.message().to_string() },
            x => { format!("unexpected response: {}",x.as_mini().description()) }
        }
    }

    #[allow(unused)] // used in debug_big_requests
    pub(super) fn description(&self) -> String { self.as_mini().description().to_string() }
    #[allow(unused)] // used in debug_big_requests
    pub(super) fn component_size(&self) -> Vec<(String,usize)> { self.as_mini().component_size() }

    accessor!(self,into_jump,Jump,JumpRes);
    accessor!(self,into_program,Program,ProgramRes);
    accessor!(self,into_stick,Stick,StickRes);
    accessor!(self,into_authority,Authority,AuthorityRes);
    accessor!(self,into_data,Data,DataRes);
    accessor!(self,into_boot_channel,BootChannel,BootChannelRes);

    #[cfg(debug_big_requests)]
    pub(crate) fn total_size(&self) -> usize { self.as_mini().total_size() }
}

struct MiniResponseVisitor;

impl<'de> Visitor<'de> for MiniResponseVisitor {
    type Value = MiniResponse;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a MiniResponse")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let opcode = st_field("opcode",seq.next_element()?)?;
        Ok(match opcode {
            0 => MiniResponse::BootChannel(st_field("opdata",seq.next_element()?)?),
            1 => MiniResponse::FailureRes(st_field("opdata",seq.next_element()?)?),
            2 => MiniResponse::Program(st_field("opdata",seq.next_element()?)?),
            3 => MiniResponse::Stick(st_field("opdata",seq.next_element()?)?),
            4 => MiniResponse::Authority(st_field("opdata",seq.next_element()?)?),
            5 => MiniResponse::Data(st_field("opdata",seq.next_element()?)?),
            6 => MiniResponse::Jump(st_field("opdata",seq.next_element()?)?),
            v => { return Err(serde::de::Error::custom(format!("unknown opcode {}",v))); }
        })
    }
}

impl<'de> Deserialize<'de> for MiniResponse {
    fn deserialize<D>(deserializer: D) -> Result<MiniResponse, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_map(MiniResponseVisitor)
    }
}

pub struct MiniResponseAttempt {
    msg_id: u64,
    variety: MiniResponse
}

impl MiniResponseAttempt {
    pub(super) fn new(msg_id: u64, variety: MiniResponse) -> MiniResponseAttempt {
        MiniResponseAttempt { msg_id, variety }
    }

    pub(crate) fn message_id(&self) -> u64 { self.msg_id }
    pub(crate) fn into_variety(self) -> MiniResponse { self.variety }

    #[allow(unused)] // used in debug_big_requests
    pub(crate) fn total_size(&self) -> usize { self.variety.as_mini().total_size() }
    #[allow(unused)] // used in debug_big_requests
    pub(super) fn description(&self) -> String { self.variety.description() }
    #[allow(unused)] // used in debug_big_requests
    pub(super) fn component_size(&self) -> Vec<(String,usize)> { self.variety.component_size() }
}

struct MiniResponseAttemptVisitor;

impl<'de> Visitor<'de> for MiniResponseAttemptVisitor {
    type Value = MiniResponseAttempt;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a MiniResponseAttempt")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de> {
        let msg_id = st_field("msgid",seq.next_element()?)?;
        let variety = st_field("variety",seq.next_element()?)?;
        Ok(MiniResponseAttempt { msg_id, variety })
    }
}

impl<'de> Deserialize<'de> for MiniResponseAttempt {
    fn deserialize<D>(deserializer: D) -> Result<MiniResponseAttempt, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(MiniResponseAttemptVisitor)
    }
}
