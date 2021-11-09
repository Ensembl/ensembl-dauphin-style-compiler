use peregrine_toolkit::{cbor::{cbor_as_number, cbor_into_vec, check_array_len}, decompose_vec};
use crate::request::messages::{authorityres::AuthorityRes, bootstrapres::BootRes, datares::DataRes, failureres::FailureRes, jumpres::JumpRes, programres::ProgramRes, stickres::StickRes};
use serde_cbor::Value as CborValue;

pub enum BackendResponse {
    Bootstrap(BootRes),
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
                BackendResponse::$branch(j) => Ok(j),
                _ => Err($self.bad_response())
            }
        }
                
    };
}

impl BackendResponse {
    fn bad_response(&self) -> String {
        let unexpected = match self {
            BackendResponse::FailureRes(g) => {
                return g.message().to_string();
            },
            BackendResponse::Bootstrap(_) => "bootstrap",
            BackendResponse::Program(_) => "program",
            BackendResponse::Stick(_) => "stick",
            BackendResponse::Authority(_) => "authority",
            BackendResponse::Data(_) => "data",
            BackendResponse::Jump(_) => "jump"
        };
        format!("unexpected response: {}",unexpected)
    }

    accessor!(self,into_jump,Jump,JumpRes);
    accessor!(self,into_program,Program,ProgramRes);
    accessor!(self,into_stick,Stick,StickRes);
    accessor!(self,into_authority,Authority,AuthorityRes);
    accessor!(self,into_data,Data,DataRes);
    accessor!(self,into_bootstrap,Bootstrap,BootRes);

    pub fn decode(value: CborValue) -> Result<BackendResponse,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,2)?;
        decompose_vec!(seq,variety,value);
        Ok(match cbor_as_number(&variety)? {
            0 => BackendResponse::Bootstrap(BootRes::decode(value)?),
            1 => BackendResponse::FailureRes(FailureRes::decode(value)?),
            2 => BackendResponse::Program(ProgramRes::decode(value)?),
            3 => BackendResponse::Stick(StickRes::decode(value)?),
            4 => BackendResponse::Authority(AuthorityRes::decode(value)?),
            5 => BackendResponse::Data(DataRes::decode(value)?),
            6 => BackendResponse::Jump(JumpRes::decode(value)?),
            v => { return Err(format!("bad response type: {}",v)) }
        })
    }
}

pub struct BackendResponseAttempt {
    msg_id: u64,
    variety: BackendResponse
}

impl BackendResponseAttempt {
    pub(super) fn new(msg_id: u64, variety: BackendResponse) -> BackendResponseAttempt {
        BackendResponseAttempt { msg_id, variety }
    }

    pub fn decode(value: CborValue) -> Result<BackendResponseAttempt,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,2)?;
        decompose_vec!(seq,msg_id,variety);
        Ok(BackendResponseAttempt {
            msg_id: cbor_as_number(&msg_id)?,
            variety: BackendResponse::decode(variety)?
        })
    }

    pub(crate) fn message_id(&self) -> u64 { self.msg_id }
    pub(crate) fn into_variety(self) -> BackendResponse { self.variety }
}
