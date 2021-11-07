use peregrine_toolkit::envaryseq;
use serde::{Deserializer, Serializer};
use crate::core::stick::{ Stick, StickId };
use super::backoff::Backoff;
use super::channel::{ Channel, PacketPriority };
use super::request::{RequestType};
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
        let r = backoff.backoff(RequestType::new_stick(self.clone()), |v| {
            v.into_stick()
        }).await?;
        Ok(r.stick)
    }
}

impl serde::Serialize for StickCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        envaryseq!(serializer,self.stick_id.get_id().to_string())
    }
}

pub struct StickCommandResponse {
    stick: Stick
}

impl<'de> serde::Deserialize<'de> for StickCommandResponse {
    fn deserialize<D>(deserializer: D) -> Result<StickCommandResponse, D::Error> where D: Deserializer<'de> {
        Ok(StickCommandResponse {
            stick: Stick::deserialize(deserializer)?
        })
    }
}

pub(super) async fn do_stick_request(mut manager: RequestManager, channel: Channel, name: StickId) -> anyhow::Result<Stick> {
    let req = StickCommandRequest::new(&name);
    req.execute(&channel,&mut manager).await
}
