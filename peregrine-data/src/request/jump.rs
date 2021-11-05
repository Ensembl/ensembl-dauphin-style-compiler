use std::any::Any;
use serde::{Deserialize};
use serde_cbor::Value as CborValue;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
struct JumpCommandRequest {
    location: String
}

impl JumpCommandRequest {
    fn new(location: &str) -> JumpCommandRequest {
        JumpCommandRequest {
            location: location.to_string()
        }
    }

    async fn execute(self, channel: &Channel, manager: &RequestManager) -> anyhow::Result<JumpResponse> {
        let mut backoff = Backoff::new(manager,channel,&PacketPriority::RealTime);
        let r = backoff.backoff::<JumpResponse,_>(self.clone()).await??;
        Ok(r.as_ref().clone())
    }
}

impl RequestType for JumpCommandRequest {
    fn type_index(&self) -> u8 { 5 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![CborValue::Text(self.location.to_string())]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("getting jump location"))
    }
}

#[derive(Clone,Deserialize)]
pub struct JumpLocation {
    pub stick: String,
    pub left: u64,
    pub right: u64
}

#[derive(Clone,Deserialize)]
struct NotFound { no: bool }

#[derive(Clone,Deserialize)]
#[serde(untagged)]
enum JumpResponse {
    Found(JumpLocation),
    NotFound(NotFound)
}

impl ResponseType for JumpResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct JumpResponseBuilderType();

impl ResponseBuilderType for JumpResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let xxx_bytes = serde_cbor::to_vec(value)?;
        Ok(Box::new(serde_cbor::from_slice::<JumpResponse>(&xxx_bytes)?))
    }
}

pub async fn do_jump_request(mut manager: RequestManager, channel: Channel, location: &str) -> anyhow::Result<Option<JumpLocation>> {
    let req = JumpCommandRequest::new(&location);
    Ok(match req.execute(&channel,&mut manager).await? {
        JumpResponse::Found(x) => Some(x),
        JumpResponse::NotFound(_) => None
    })
}
