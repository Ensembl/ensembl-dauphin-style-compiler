use std::any::Any;
use anyhow::{ self, anyhow as err, bail };
use blackbox::{ blackbox_count, blackbox_log };
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_string, cbor_map, cbor_map_iter };
use crate::run::pgcommander::{ PgCommander, PgCommanderTaskSpec };
use crate::run::PgDauphin;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::program::ProgramLoader;
use super::request::{ RequestType, ResponseType, ResponseBuilderType, CommandResponse };
use super::packet::{ ResponsePacketBuilderBuilder };
use super::backoff::Backoff;

#[derive(Clone)]
pub struct BootstrapCommandRequest {
    commander: PgCommander,
    dauphin: PgDauphin,
    loader: ProgramLoader,
    channel: Channel
}

impl BootstrapCommandRequest {
    fn new(commander: &PgCommander, dauphin: &PgDauphin, loader: &ProgramLoader, channel: Channel) -> BootstrapCommandRequest {
        BootstrapCommandRequest {
            commander: commander.clone(),
            dauphin: dauphin.clone(),
            loader: loader.clone(),
            channel
        }
    }

    async fn execute(self, mut manager: RequestManager) -> anyhow::Result<()> {
        blackbox_log!(&format!("channel-{}",self.channel.to_string()),"issuing bootstrap request");
        blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-request",1);
        let commander = self.commander.clone();
        let dauphin = self.dauphin.clone();
        let loader = self.loader.clone();
        let mut backoff = Backoff::new();
        match backoff.backoff_two_messages::<BootstrapCommandResponse,BootstrapFailure,_>(
                                    &mut manager,self.clone(),&self.channel,PacketPriority::RealTime).await? {
            Ok(b) => {
                blackbox_log!(&format!("channel-{}",self.channel.to_string()),"bootstrap response received");
                blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-response-success",1);
                Ok(b.bootstrap(&dauphin,&loader,&commander).await?)
            }
            Err(_) => {
                blackbox_count!(&format!("channel-{}",self.channel.to_string()),"bootstrap-response-fail",1);
                bail!("failed to bootstrap to '{}'. backend missing?",self.channel);
            }
        }
    }
}

impl RequestType for BootstrapCommandRequest {
    fn type_index(&self) -> u8 { 0 }
    fn serialize(&self) -> anyhow::Result<CborValue> { Ok(CborValue::Null) }
    fn to_failure(&self) -> Box<dyn ResponseType> { Box::new(BootstrapFailure{}) }
}

pub struct BootstrapCommandResponse {
    channel: Channel,
    name: String // in-channel name
}

impl ResponseType for BootstrapCommandResponse {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

impl BootstrapCommandResponse {
    async fn bootstrap(&self, dauphin: &PgDauphin, loader: &ProgramLoader, commander: &PgCommander) -> anyhow::Result<()> {
        let dauphin_prog = dauphin.load_program(loader,&self.channel,&self.name).await?;
        commander.add_task(PgCommanderTaskSpec {
            name: "dauphin: bootstrap".to_string(),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(dauphin_prog.run())
        });
        Ok(())
    }
}

pub struct BootstrapResponseBuilderType();
impl ResponseBuilderType for BootstrapResponseBuilderType {
    fn deserialize(&self, value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        let values = cbor_array(value,2,false)?;
        Ok(Box::new(BootstrapCommandResponse {
            channel: Channel::deserialize(&values[0])?,
            name: cbor_string(&values[1])?
        }))
    }
}

pub struct BootstrapFailure {
}

impl ResponseType for BootstrapFailure {
    fn as_any(&self) -> &dyn Any { self }
    fn into_any(self: Box<Self>) -> Box<dyn Any> { self }
}

pub struct BootstrapFailureBuilderType();
impl ResponseBuilderType for BootstrapFailureBuilderType {
    fn deserialize(&self, _value: &CborValue) -> anyhow::Result<Box<dyn ResponseType>> {
        Ok(Box::new(BootstrapFailure{}))
    }
}

pub fn bootstrap(requests: &RequestManager, loader: &ProgramLoader, commander: &PgCommander, dauphin: &PgDauphin, channel: Channel) -> anyhow::Result<()> {
    let req = BootstrapCommandRequest::new(commander,dauphin,loader,channel);
    let boot_proc = req.execute(requests.clone());
    commander.add_task(PgCommanderTaskSpec {
        name: "bootstrap".to_string(),
        prio: 4,
        slot: None,
        timeout: None,
        task: Box::pin(boot_proc)
    });
    Ok(())
}

pub(super) fn bootstrap_commands(rspbb: &mut ResponsePacketBuilderBuilder) {
    rspbb.register(0,Box::new(BootstrapResponseBuilderType()));
    rspbb.register(1,Box::new(BootstrapFailureBuilderType()));
}
