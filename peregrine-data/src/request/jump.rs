use std::any::Any;
use peregrine_toolkit::envaryseq;
use serde::{Deserialize, Serializer};
use serde_cbor::Value as CborValue;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::request::{RequestType, ResponseBuilderType, ResponseType};
use super::manager::RequestManager;

#[derive(Clone)]
pub struct JumpCommandRequest {
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
        let r = backoff.backoff_new::<JumpResponse>(RequestType::new_jump(self.clone())).await??;
        Ok(r.as_ref().clone())
    }
}

impl serde::Serialize for JumpCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        envaryseq!(serializer,self.location.to_string())
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
