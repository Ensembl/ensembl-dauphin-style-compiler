use std::sync::{ Arc };
use crate::core::stick::{ Stick, StickId };
use crate::index::StickAuthorityStore;
use crate::run::{ PgCommander, add_task };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::memoized::Memoized;
use crate::CountingPromise;

#[derive(Clone)]
pub struct StickStore {
    booted: CountingPromise,
    store: Memoized<StickId,Option<Stick>>
}

impl StickStore {
    pub fn new(commander: &PgCommander, sas: &StickAuthorityStore, booted: &CountingPromise) -> StickStore {
        let commander = commander.clone();
        let sas = sas.clone();
        StickStore {
            booted: booted.clone(),
            store: Memoized::new(move |stick_id: &StickId, result| {
                let stick_id = stick_id.clone();
                let sas = sas.clone();
                add_task(&commander,PgCommanderTaskSpec {
                    name: format!("stick-loader-{}",stick_id.get_id()),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(async move {
                        sas.try_lookup(stick_id).await.unwrap_or(());
                        result.resolve(None);
                        Ok(())
                    })
                });
            })
        }
    }

    pub fn add(&self, key: StickId, value: Option<Stick>) {
        self.store.add(key,value);
    }

    pub async fn get(&self, key: &StickId) -> Arc<Option<Stick>> {
        self.booted.wait().await;
        self.store.get(key).await
    }
}
