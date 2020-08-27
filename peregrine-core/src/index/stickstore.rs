use crate::lock;
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use crate::core::stick::{ Stick, StickId, StickTopology };
use anyhow::bail;
use crate::index::StickAuthorityStore;
use crate::request::channel::{ Channel, PacketPriority, ChannelIntegration };
use crate::request::manager::{ RequestManager, PayloadReceiver };
use crate::request::packet::ResponsePacket;
use crate::request::stick::issue_stick_request;
use crate::run::{ PgCommander, PgDauphin };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::memoized::Memoized;

#[derive(Clone)]
pub struct StickStore {
    store: Memoized<StickId,Option<Stick>>
}

impl StickStore {
    pub fn new(commander: &PgCommander, sas: &StickAuthorityStore) -> anyhow::Result<StickStore> {
        let commander = commander.clone();
        let sas = sas.clone();
        Ok(StickStore {
            store: Memoized::new(move |stick_id: &StickId, result| {
                let stick_id = stick_id.clone();
                let sas = sas.clone();
                commander.add_task(PgCommanderTaskSpec {
                    name: format!("stick-loader-{}",stick_id.get_id()),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(async move {
                        sas.try_lookup(stick_id).await.unwrap_or(());
                        result.resolve(None);
                        Ok(())
                    })
                })
            })
        })
    }

    pub fn add(&self, key: StickId, value: Option<Stick>) {
        self.store.add(key,value);
    }

    pub async fn get(&self, key: &StickId) -> anyhow::Result<Arc<Option<Stick>>> {
        self.store.get(key).await
    }
}
