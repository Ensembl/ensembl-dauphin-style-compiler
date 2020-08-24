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
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::manager::RequestManager;
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;

#[derive(Clone)]
struct StickCommandRequest {
    stick_id: StickId
}

impl StickCommandRequest {
    pub(crate) fn new(stick_id: &StickId) -> StickCommandRequest {
        blackbox_log!("stick","requesting stick {}",stick_id.get_id());
        StickCommandRequest {
            stick_id: stick_id.clone()
        }
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> anyhow::Result<()> {
        let mut backoff = Backoff::new();
        backoff.backoff::<StickCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok(())
    }
}

impl RequestType for StickCommandRequest {
    fn type_index(&self) -> u8 { 2 }
    fn serialize(&self) -> anyhow::Result<CborValue> {
        Ok(CborValue::Array(vec![CborValue::Text(self.stick_id.get_id().to_string())]))
    }
    fn to_failure(&self) -> Rc<dyn ResponseType> {
        Rc::new(GeneralFailure::new("loading stick info failed"))
    }
}

struct StickCommandResponse {}

impl ResponseType for StickCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct StickResponseBuilderType();

impl ResponseBuilderType for StickResponseBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Rc<dyn ResponseType>> {
        Ok(Rc::new(StickCommandResponse {}))
    }
}

pub async fn get_stick(mut manager: RequestManager, channel: Channel, name: StickId) -> anyhow::Result<()> {
    let req = StickCommandRequest::new(&name);
    req.execute(&channel,&mut manager).await
}
