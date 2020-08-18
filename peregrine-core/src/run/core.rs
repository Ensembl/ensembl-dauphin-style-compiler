use anyhow;
use super::pgcommander::{ PgCommander, PgCommanderTaskSpec, Commander };
use super::pgdauphin::{ PgDauphin, PgDauphinIntegration };
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use commander::{ RunSlot };
use serde_cbor::Value as CborValue;
use crate::request::bootstrap::bootstrap;
use crate::request::manager::RequestManager;
use crate::request::packet::ResponsePacket;
use crate::request::program::ProgramLoader;
use crate::request::channel::Channel;

pub struct PgCore {
    loader: ProgramLoader,
    manager: RequestManager,
    commander: PgCommander,
    dauphin: PgDauphin
}

impl PgCore {
    pub fn new(commander: &PgCommander, dauphin: &PgDauphin, manager: &RequestManager) -> anyhow::Result<PgCore> {
        let loader = ProgramLoader::new(&commander,manager)?;
        Ok(PgCore {
            loader,
            manager: manager.clone(),
            commander: commander.clone(),
            dauphin: dauphin.clone()
        })
    }

    pub fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>) {
        self.commander.add_task(PgCommanderTaskSpec {
            name: name.to_string(),
            prio,slot,timeout,
            task: f
        })
    }

    pub fn run(&mut self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>) -> anyhow::Result<()> {
        let (dauphin,commander) = (&mut self.dauphin, &mut self.commander);
        let task = dauphin.load("test",name)?;
        commander.add_task(PgCommanderTaskSpec {
            name: format!("dauphin: '{}'",name),
            prio,slot,timeout,
            task: Box::pin(task.run())
        });
        Ok(())
    }

    pub fn add_binary(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        self.dauphin.add_binary("test",cbor)
    }

    pub fn bootstrap(&self, channel: Channel) -> anyhow::Result<()> {
        bootstrap(&self.manager,&self.loader,&self.commander,&self.dauphin,channel)
    }
}