use anyhow::bail;
use std::any::Any;
use serde_cbor::Value as CborValue;
use crate::core::stick::{ Stick, StickId, StickTopology };
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map, cbor_int };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::switch::allotment::AllotmentRequest;
use crate::util::message::DataMessage;

#[derive(Clone)]
pub(crate) struct JumpCommandRequest {
    location: String
}

impl JumpCommandRequest {
    pub(crate) fn new(location: &str) -> JumpCommandRequest {
        JumpCommandRequest {
            location: location.to_string()
        }
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> anyhow::Result<(String,u64,u64)> {
        let mut backoff = Backoff::new();
        let r = backoff.backoff::<JumpCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok((r.stick.to_string(),r.start,r.end))
    }
}

impl RequestType for JumpCommandRequest {
    fn type_index(&self) -> u8 { 5 }
    fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![CborValue::Text(self.location.to_string())]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("getting jump location"))
    }
}

struct JumpCommandResponse {
    stick: String,
    start: u64,
    end: u64
}

impl ResponseType for JumpCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct JumpResponseBuilderType();

impl ResponseBuilderType for JumpResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {

        let values = cbor_map(value,&["stick","left","right"])?;
        let stick = cbor_string(&values[0])?; 
        let start = cbor_int(&values[1],None)? as u64;
        let end = cbor_int(&values[2],None)? as u64;
        Ok(Box::new(JumpCommandResponse { stick, start, end }))
    }
}

pub async fn issue_jump_request(mut manager: RequestManager, channel: Channel, location: &str) -> anyhow::Result<(String,u64,u64)> {
    let req = JumpCommandRequest::new(&location);
    req.execute(&channel,&mut manager).await
}
