use anyhow;
use super::pgcommander::{ PgCommander, PgCommanderTaskSpec };
use super::pgdauphin::{ PgDauphin };
use std::future::Future;
use std::pin::Pin;
use commander::{ RunSlot };
use serde_cbor::Value as CborValue;
use crate::request::bootstrap::bootstrap;
use crate::request::manager::RequestManager;
use crate::request::program::ProgramLoader;
use crate::request::channel::Channel;
use crate::index::StickAuthorityStore;

#[derive(Clone)]
pub struct PgCore {
    // XXX pub
    pub loader: ProgramLoader,
    pub stick_authority_store: StickAuthorityStore,
    pub manager: RequestManager,
    pub commander: PgCommander,
    pub dauphin: PgDauphin
}

impl PgCore {
    pub fn new(commander: &PgCommander, dauphin: &PgDauphin, manager: &RequestManager, sas: &StickAuthorityStore) -> anyhow::Result<PgCore> {
        let loader = ProgramLoader::new(&commander,manager,&dauphin)?;
        Ok(PgCore {
            loader,
            manager: manager.clone(),
            commander: commander.clone(),
            dauphin: dauphin.clone(),
            stick_authority_store: sas.clone()
        })
    }

    pub fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>) {
        self.commander.add_task(PgCommanderTaskSpec {
            name: name.to_string(),
            prio,slot,timeout,
            task: f
        })
    }

    pub fn bootstrap(&self, channel: Channel) -> anyhow::Result<()> {
        bootstrap(&self.manager,&self.loader,&self.commander,&self.dauphin,channel)
    }
}