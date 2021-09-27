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
use crate::util::message::DataMessage;

#[derive(Clone)]
struct StickCommandRequest {
    stick_id: StickId
}

impl StickCommandRequest {
    pub(crate) fn new(stick_id: &StickId) -> StickCommandRequest {
        StickCommandRequest {
            stick_id: stick_id.clone()
        }
    }

    pub(crate) async fn execute(self, channel: &Channel, manager: &mut RequestManager) -> anyhow::Result<Stick> {
        let mut backoff = Backoff::new();
        let r = backoff.backoff::<StickCommandResponse,_,_>(manager,self.clone(),channel,PacketPriority::RealTime, |_| None).await??;
        Ok(r.stick.clone())
    }
}

impl RequestType for StickCommandRequest {
    fn type_index(&self) -> u8 { 2 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![CborValue::Text(self.stick_id.get_id().to_string())]))
    }
    fn to_failure(&self) -> Box<dyn ResponseType> {
        Box::new(GeneralFailure::new("loading stick info failed"))
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

pub async fn issue_stick_request(mut manager: RequestManager, channel: Channel, name: StickId) -> anyhow::Result<Stick> {
    let req = StickCommandRequest::new(&name);
    req.execute(&channel,&mut manager).await
}
