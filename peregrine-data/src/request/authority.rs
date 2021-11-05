use std::any::Any;
use serde::Serializer;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use crate::index::stickauthority::Authority;
use super::request::{RequestType, ResponseBuilderType, ResponseType};
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
pub(super) struct AuthorityCommandRequest {}

impl AuthorityCommandRequest {
    fn new() -> AuthorityCommandRequest {
        AuthorityCommandRequest {}
    }

    async fn execute(self, channel: &Channel, manager: &RequestManager) -> Result<Authority,DataMessage> {
        let mut backoff = Backoff::new(manager,channel,&PacketPriority::RealTime);
        let response = backoff.backoff_new::<AuthorityCommandResponse>(RequestType::new_authority(self.clone())).await??;
        Ok(Authority::new(&response.channel,&response.startup_name,&response.lookup_name,&response.jump_name))
    }
}

impl serde::Serialize for AuthorityCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}

struct AuthorityCommandResponse {
    channel: Channel,
    startup_name: String,
    lookup_name: String,
    jump_name: String
}

impl ResponseType for AuthorityCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct AuthorityResponseBuilderType();

impl ResponseBuilderType for AuthorityResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_array(value,4,false)?;
        let channel = Channel::deserialize(&values[0])?;
        let startup_name = cbor_string(&values[1])?;
        let lookup_name = cbor_string(&values[2])?;
        let jump_name = cbor_string(&values[3])?;
        Ok(Box::new(AuthorityCommandResponse {
            channel,
            startup_name: startup_name.to_string(),
            lookup_name: lookup_name.to_string(),
            jump_name: jump_name.to_string()
        }))
    }
}

pub(super) async fn do_stick_authority(mut manager: RequestManager, channel: Channel) -> Result<Authority,DataMessage> {
    let req = AuthorityCommandRequest::new();
    req.execute(&channel,&mut manager).await
}
