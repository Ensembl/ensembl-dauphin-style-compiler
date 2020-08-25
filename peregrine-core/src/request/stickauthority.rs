use anyhow::bail;
use std::any::Any;
use std::collections::{ HashMap };
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use blackbox::blackbox_log;
use serde_cbor::Value as CborValue;
use crate::core::stick::{ Stick, StickId };
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map_iter };
use crate::util::singlefile::SingleFile;
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use crate::index::stickauthority::StickAuthority;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;

#[derive(Clone)]
struct StickAuthorityCommandRequest {}

impl StickAuthorityCommandRequest {
    pub(crate) fn new() -> StickAuthorityCommandRequest {
        StickAuthorityCommandRequest {}
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> anyhow::Result<StickAuthority> {
        let mut backoff = Backoff::new();
        blackbox_log!("stickauthority","registering authority at {}",channel.to_string());
        let response = backoff.backoff::<StickAuthorityCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok(StickAuthority::new(&response.channel,&response.startup_name,&response.lookup_name))
    }
}

impl RequestType for StickAuthorityCommandRequest {
    fn type_index(&self) -> u8 { 3 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Null)
    }
    fn to_failure(&self) -> Rc<dyn ResponseType> {
        Rc::new(GeneralFailure::new("loading stick info failed"))
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
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Rc<dyn ResponseType>> {
        let values = cbor_array(value,3,false)?;
        let channel = Channel::deserialize(&values[0])?;
        let startup_name = cbor_string(&values[1])?;
        let lookup_name = cbor_string(&values[2])?;
        Ok(Rc::new(StickAuthorityCommandResponse {
            channel,
            startup_name: startup_name.to_string(),
            lookup_name: lookup_name.to_string()
        }))
    }
}

pub async fn get_stick_authority(mut manager: RequestManager, channel: Channel) -> anyhow::Result<StickAuthority> {
    let req = StickAuthorityCommandRequest::new();
    req.execute(&channel,&mut manager).await
}
