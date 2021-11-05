use std::any::Any;
use std::sync::Arc;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_string };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use crate::index::stickauthority::StickAuthority;
use super::request::{OldRequestType, ResponseBuilderType, ResponseType};
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
struct StickAuthorityCommandRequest {}

impl StickAuthorityCommandRequest {
    fn new() -> StickAuthorityCommandRequest {
        StickAuthorityCommandRequest {}
    }

    async fn execute(self, channel: &Channel, manager: &RequestManager) -> Result<StickAuthority,DataMessage> {
        let mut backoff = Backoff::new(manager,channel,&PacketPriority::RealTime);
        let response = backoff.backoff_old::<_,StickAuthorityCommandResponse>(self.clone()).await??;
        Ok(StickAuthority::new(&response.channel,&response.startup_name,&response.lookup_name,&response.jump_name))
    }
}

impl OldRequestType for StickAuthorityCommandRequest {
    fn type_index(&self) -> u8 { 3 }
    fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Null)
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("loading stick info failed"))
    }
}

struct StickAuthorityCommandResponse {
    channel: Channel,
    startup_name: String,
    lookup_name: String,
    jump_name: String
}

impl ResponseType for StickAuthorityCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct StickAuthorityResponseBuilderType();

impl ResponseBuilderType for StickAuthorityResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_array(value,4,false)?;
        let channel = Channel::deserialize(&values[0])?;
        let startup_name = cbor_string(&values[1])?;
        let lookup_name = cbor_string(&values[2])?;
        let jump_name = cbor_string(&values[3])?;
        Ok(Box::new(StickAuthorityCommandResponse {
            channel,
            startup_name: startup_name.to_string(),
            lookup_name: lookup_name.to_string(),
            jump_name: jump_name.to_string()
        }))
    }
}

pub(super) async fn do_stick_authority(mut manager: RequestManager, channel: Channel) -> Result<StickAuthority,DataMessage> {
    let req = StickAuthorityCommandRequest::new();
    req.execute(&channel,&mut manager).await
}
