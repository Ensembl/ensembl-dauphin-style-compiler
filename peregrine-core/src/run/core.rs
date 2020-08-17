use anyhow;
use super::pgcommander::PgCommander;
use super::pgdauphin::{ PgDauphin, PgDauphinIntegration };
use std::future::Future;
use std::pin::Pin;
use commander::{ RunSlot };
use serde_cbor::Value as CborValue;
use crate::request::request::ResponsePacket;

pub struct PgCore {
    commander: Box<dyn PgCommander>,
    dauphin: PgDauphin
}

impl PgCore {
    pub fn new(commander: Box<dyn PgCommander>, dauphin_integration: Box<dyn PgDauphinIntegration>) -> anyhow::Result<PgCore> {
        Ok(PgCore {
            commander,
            dauphin: PgDauphin::new(dauphin_integration)?
        })
    }

    pub fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>) {
        self.commander.add_task(name,prio,slot,timeout,f)
    }

    pub fn run(&mut self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>) -> anyhow::Result<()> {
        let (dauphin,commander) = (&mut self.dauphin, &mut self.commander);
        let task = dauphin.load("test",name)?;
        commander.add_task(&format!("dauphin: '{}'",name),prio,slot,timeout,Box::pin(task.run()));
        Ok(())
    }

    pub fn add_binary(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        self.dauphin.add_binary("test",cbor)
    }

    pub fn process_response(&mut self, response: &ResponsePacket) -> anyhow::Result<()> {
        self.dauphin.add_programs(response)?;
        Ok(())
    }
}