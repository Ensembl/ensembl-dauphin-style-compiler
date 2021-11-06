use peregrine_toolkit::envaryseq;
use serde::{Serializer};
use serde_derive::Deserialize;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::request::RequestType;
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
        let r = backoff.backoff(RequestType::new_jump(self.clone()), |v| {
            v.into_jump()
        }).await?;
        Ok(r)
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
pub struct NotFound { no: bool }

#[derive(Clone,Deserialize)]
#[serde(untagged)]
pub enum JumpResponse {
    Found(JumpLocation),
    NotFound(NotFound)
}

pub async fn do_jump_request(mut manager: RequestManager, channel: Channel, location: &str) -> anyhow::Result<Option<JumpLocation>> {
    let req = JumpCommandRequest::new(&location);
    Ok(match req.execute(&channel,&mut manager).await? {
        JumpResponse::Found(x) => Some(x),
        JumpResponse::NotFound(_) => None
    })
}
