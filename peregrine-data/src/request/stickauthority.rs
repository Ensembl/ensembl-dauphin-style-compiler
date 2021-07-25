use std::any::Any;
use serde_cbor::Value as CborValue;
use crate::core::stick::{ Stick, StickId };
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map_iter };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use crate::index::stickauthority::StickAuthority;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::util::message::DataMessage;

#[derive(Clone)]
struct StickAuthorityCommandRequest {}

impl StickAuthorityCommandRequest {
    pub(crate) fn new() -> StickAuthorityCommandRequest {
        StickAuthorityCommandRequest {}
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> Result<StickAuthority,DataMessage> {
        let mut backoff = Backoff::new();
        let response = backoff.backoff::<StickAuthorityCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok(StickAuthority::new(&response.channel,&response.startup_name,&response.lookup_name))
    }
}

impl RequestType for StickAuthorityCommandRequest {
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
    lookup_name: String
}

impl ResponseType for StickAuthorityCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct StickAuthorityResponseBuilderType();

impl ResponseBuilderType for StickAuthorityResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_array(value,3,false)?;
        let channel = Channel::deserialize(&values[0])?;
        let startup_name = cbor_string(&values[1])?;
        let lookup_name = cbor_string(&values[2])?;
        Ok(Box::new(StickAuthorityCommandResponse {
            channel,
            startup_name: startup_name.to_string(),
            lookup_name: lookup_name.to_string()
        }))
    }
}

pub async fn get_stick_authority(mut manager: RequestManager, channel: Channel) -> Result<StickAuthority,DataMessage> {
    let req = StickAuthorityCommandRequest::new();
    req.execute(&channel,&mut manager).await
}
