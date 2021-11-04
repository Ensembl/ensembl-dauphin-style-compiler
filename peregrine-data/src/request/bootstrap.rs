use crate::core::asset::AssetsBuilder;
use crate::core::Assets;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use anyhow::{ self, };
use serde_cbor::Value as CborValue;
use crate::core::Asset;
use crate::{ChannelLocation, PeregrineIntegration};
use crate::run::pgcommander::{ PgCommanderTaskSpec };
use crate::run::{ PgDauphin, PgDauphinTaskSpec, add_task };
use super::channel::{ Channel, PacketPriority };
use super::failure::GeneralFailure;
use super::manager::RequestManager;
use super::program::ProgramLoader;
use super::request::{ RequestType, ResponseType, ResponseBuilderType };
use super::backoff::Backoff;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore, PeregrineApiQueue, ApiMessage };
use crate::lane::programname::ProgramName;
use crate::util::cbor::{cbor_bytes, cbor_map, cbor_map_iter, cbor_map_key, cbor_string};

#[derive(Clone)]
pub struct BootstrapCommandRequest {
    dauphin: PgDauphin,
    queue: PeregrineApiQueue,
    loader: ProgramLoader,
    channel: Channel
}

impl BootstrapCommandRequest {
    fn new(dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader, channel: Channel) -> BootstrapCommandRequest {
        BootstrapCommandRequest {
            dauphin: dauphin.clone(),
            queue: queue.clone(),
            loader: loader.clone(),
            channel
        }
    }

    async fn execute(self, manager: &RequestManager, integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>) -> Result<(),DataMessage> {
        let dauphin = self.dauphin.clone();
        let loader = self.loader.clone();
        let mut backoff = Backoff::new(&manager,&self.channel,&PacketPriority::RealTime);
        match backoff.backoff::<BootstrapCommandResponse,_>(self.clone()).await? {
            Ok(b) => {
                manager.set_lo_divert(&b.channel_hi,&b.channel_lo);
                b.bootstrap(&dauphin,&self.queue,&loader,integration).await?;
                Ok(())
            }
            Err(e) => {
                Err(DataMessage::BadBootstrapCannotStart(self.channel.clone(),Box::new(e.clone())))
            }
        }
    }
}

impl RequestType for BootstrapCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self, _channel: &Channel) -> Result<CborValue,DataMessage> { Ok(CborValue::Null) }
    fn to_failure(&self) -> Box<dyn ResponseType> { Box::new(GeneralFailure::new("bootstrap failed")) }
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
    async fn bootstrap(&self, dauphin: &PgDauphin, queue: &PeregrineApiQueue, loader: &ProgramLoader, integration: &Arc<Mutex<Box<dyn PeregrineIntegration>>>) -> Result<(),DataMessage> {
        dauphin.run_program(loader,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.program_name.clone(),
            payloads: None
        }).await?;
        integration.lock().unwrap().set_assets(self.assets.clone()); // XXX don't clone
        queue.push(ApiMessage::SetAssets(self.assets.clone()));
        queue.push(ApiMessage::RegeneraateTrackConfig);
        Ok(())
    }
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

pub(crate) fn bootstrap(base: &PeregrineCoreBase, agent_store: &AgentStore, channel: Channel, identity: u64) {
    *base.identity.lock().unwrap() = identity;
    let base2 = base.clone();
    let agent_store = agent_store.clone();
    add_task(&base.commander,PgCommanderTaskSpec {
        name: "bootstrap".to_string(),
        prio: 4,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            let req = BootstrapCommandRequest::new(&base2.dauphin,&base2.queue,&agent_store.program_loader,channel.clone());
            let r = req.execute(&base2.manager,&base2.integration).await;
            let r = r.unwrap_or(());
            base2.booted.unlock();
            Ok(())
        }),
        stats: false
    });
}
