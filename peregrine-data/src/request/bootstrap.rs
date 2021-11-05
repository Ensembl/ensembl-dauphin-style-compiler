use crate::core::asset::AssetsBuilder;
use crate::core::Assets;
use std::any::Any;
use anyhow::{ self, };
use serde::Serializer;
use serde_cbor::Value as CborValue;
use crate::core::Asset;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::request::{NewRequestType, ResponseBuilderType, ResponseType};
use super::backoff::Backoff;
use crate::util::message::DataMessage;
use crate::lane::programname::ProgramName;
use crate::util::cbor::{cbor_map, cbor_map_iter, cbor_map_key, cbor_string};

#[derive(Clone)]
pub(super) struct BootstrapCommandRequest {
    channel: Channel
}

pub(super) async fn do_bootstrap(manager: &RequestManager, channel: &Channel) -> Result<Box<BootstrapCommandResponse>,DataMessage> {
    let request = BootstrapCommandRequest::new(channel.clone());
    let mut backoff = Backoff::new(&manager,&channel,&PacketPriority::RealTime);
    match backoff.backoff_new::<BootstrapCommandResponse>(NewRequestType::new_bootstrap(request)).await? {
        Ok(response) => {
            Ok(response)
        }
        Err(e) => {
            Err(DataMessage::BadBootstrapCannotStart(channel.clone(),Box::new(e.clone())))
        }
    }
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

pub struct BootstrapCommandResponse {
    program_name: ProgramName,
    channel_hi: Channel,
    channel_lo: Channel,
    assets: Assets
}

impl ResponseType for BootstrapCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl BootstrapCommandResponse {
    pub(crate) fn program_name(&self) -> &ProgramName { &self.program_name }
    pub(crate) fn assets(&self) -> &Assets { &self.assets }
    pub(crate) fn channel_hi(&self) -> &Channel { &self.channel_hi }
    pub(crate) fn channel_lo(&self) -> &Channel { &self.channel_lo }
}

pub struct BootstrapResponseBuilderType();
impl ResponseBuilderType for BootstrapResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_map(value,&["boot","hi","lo"])?;
        let channel_hi = Channel::deserialize(&values[1])?;
        let channel_lo = Channel::deserialize(&values[2])?;
        let mut assets = AssetsBuilder::new();
        if let Some(assets_in) = cbor_map_key(value,"assets")? {
            for (name,asset) in cbor_map_iter(assets_in)? {
                let name = cbor_string(name)?;
                assets.insert(&name,Asset::new(asset)?);
            }
        }
        Ok(Box::new(BootstrapCommandResponse {
            program_name: ProgramName::deserialize(&values[0])?,
            channel_hi, channel_lo, assets: assets.build()
        }))
    }
}
