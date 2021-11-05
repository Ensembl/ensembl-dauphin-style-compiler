use anyhow::bail;
use peregrine_toolkit::envaryseq;
use serde::Serializer;
use std::any::Any;
use serde_cbor::Value as CborValue;
use crate::core::stick::{ Stick, StickId, StickTopology };
use crate::util::cbor::{ cbor_array, cbor_string, cbor_map, cbor_int };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::request::{NewRequestType, ResponseBuilderType, ResponseType};
use super::manager::RequestManager;

#[derive(Clone)]
pub(super) struct StickCommandRequest {
    stick_id: StickId
}

impl StickCommandRequest {
    fn new(stick_id: &StickId) -> StickCommandRequest {
        StickCommandRequest {
            stick_id: stick_id.clone()
        }
    }

    async fn execute(self, channel: &Channel, manager: &RequestManager) -> anyhow::Result<Stick> {
        let mut backoff = Backoff::new(manager,channel,&PacketPriority::RealTime);
        let r = backoff.backoff_new::<StickCommandResponse>(NewRequestType::new_stick(self.clone())).await??;
        Ok(r.stick.clone())
    }
}

impl serde::Serialize for StickCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        envaryseq!(serializer,self.stick_id.get_id().to_string())
    }
}

struct StickCommandResponse {
    stick: Stick
}

impl ResponseType for StickCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct StickResponseBuilderType();

impl ResponseBuilderType for StickResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_map(value,&["id","size","topology","tags"])?;
        let size = cbor_int(&values[1],None)? as u64;
        let topology = match cbor_int(&values[2],None)? {
            0 => StickTopology::Linear,
            1 => StickTopology::Circular,
            _ => bail!("bad packet (stick topology)")
        };
        let tags : anyhow::Result<Vec<String>> = cbor_array(&values[3],0,true)?.iter().map(|x| cbor_string(x)).collect();
        Ok(Box::new(StickCommandResponse { stick: Stick::new(&StickId::new(&cbor_string(&values[0])?),size,topology,&tags?) })) // XXX
    }
}

pub(super) async fn do_stick_request(mut manager: RequestManager, channel: Channel, name: StickId) -> anyhow::Result<Stick> {
    let req = StickCommandRequest::new(&name);
    req.execute(&channel,&mut manager).await
}
