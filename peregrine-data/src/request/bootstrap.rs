use crate::core::Assets;
use serde::{Serializer};
use serde_derive::Deserialize;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::request::{RequestType};
use super::backoff::Backoff;
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;

#[derive(Clone)]
pub(super) struct BootstrapCommandRequest {
    channel: Channel
}

pub(super) async fn do_bootstrap(manager: &RequestManager, channel: &Channel) -> Result<BootstrapCommandResponse,DataMessage> {
    let request = BootstrapCommandRequest::new(channel.clone());
    let mut backoff = Backoff::new(&manager,&channel,&PacketPriority::RealTime);
    backoff.backoff(RequestType::new_bootstrap(request.clone()), |v| {
        v.into_bootstrap()
    }).await
}

impl BootstrapCommandRequest {
    fn new(channel: Channel) -> BootstrapCommandRequest {
        BootstrapCommandRequest {
            channel
        }
    }
}

impl serde::Serialize for BootstrapCommandRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_none()
    }
}

#[derive(Deserialize)]
pub struct BootstrapCommandResponse {
    #[serde(alias = "boot")]
    program_name: ProgramName,
    #[serde(alias = "hi")]
    channel_hi: Channel,
    #[serde(alias = "lo")]
    channel_lo: Channel,
    #[serde(default = "Assets::empty")]
    assets: Assets,
}

impl BootstrapCommandResponse {
    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn assets(&self) -> &Assets { &self.assets }
    pub(crate) fn channel_hi(&self) -> &Channel { &self.channel_hi }
    pub(crate) fn channel_lo(&self) -> &Channel { &self.channel_lo }
}
