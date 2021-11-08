use peregrine_toolkit::{cbor::{cbor_as_number, cbor_into_vec, check_array_len}, decompose_vec};
use crate::request::messages::{authorityres::AuthorityCommandResponse, bootstrapres::BootstrapCommandResponse, datares::DataResponse, failureres::GeneralFailure, jumpres::JumpResponse, programres::ProgramCommandResponse, stickres::StickCommandResponse};
use serde_cbor::Value as CborValue;

pub struct NewCommandResponse {
    msg_id: u64,
    variety: NewResponse
}

impl NewCommandResponse {
    pub(super) fn new(msg_id: u64, variety: NewResponse) -> NewCommandResponse {
        NewCommandResponse { msg_id, variety }
    }

    pub fn decode(value: CborValue) -> Result<NewCommandResponse,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,2)?;
        decompose_vec!(seq,msg_id,variety);
        Ok(NewCommandResponse {
            msg_id: cbor_as_number(&msg_id)?,
            variety: NewResponse::decode(variety)?
        })
    }

    pub(crate) fn message_id(&self) -> u64 { self.msg_id }
    pub(crate) fn into_variety(self) -> NewResponse { self.variety }
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

macro_rules! accessor {
    ($self:ident,$name:tt,$branch:tt,$result:ty) => {
        pub(crate) fn $name($self) -> Result<$result,String> {
            match $self {
                NewResponse::$branch(j) => Ok(j),
                _ => Err($self.bad_response())
            }
        }
                
    };
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

    accessor!(self,into_jump,Jump,JumpResponse);
    accessor!(self,into_program,Program,ProgramCommandResponse);
    accessor!(self,into_stick,Stick,StickCommandResponse);
    accessor!(self,into_authority,Authority,AuthorityCommandResponse);
    accessor!(self,into_data,Data,DataResponse);
    accessor!(self,into_bootstrap,Bootstrap,BootstrapCommandResponse);

    pub fn decode(value: CborValue) -> Result<NewResponse,String> {
        let mut seq = cbor_into_vec(value)?;
        check_array_len(&seq,2)?;
        decompose_vec!(seq,variety,value);
        Ok(match cbor_as_number(&variety)? {
            0 => NewResponse::Bootstrap(BootstrapCommandResponse::decode(value)?),
            1 => NewResponse::GeneralFailure(GeneralFailure::decode(value)?),
            2 => NewResponse::Program(ProgramCommandResponse::decode(value)?),
            3 => NewResponse::Stick(StickCommandResponse::decode(value)?),
            4 => NewResponse::Authority(AuthorityCommandResponse::decode(value)?),
            5 => NewResponse::Data(DataResponse::decode(value)?),
            6 => NewResponse::Jump(JumpResponse::decode(value)?),
            v => { return Err(format!("bad response type: {}",v)) }
        })
    }
}
