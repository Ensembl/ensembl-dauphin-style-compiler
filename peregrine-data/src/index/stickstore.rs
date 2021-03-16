use std::sync::{ Arc, Mutex };
use std::collections::HashMap;
use crate::{DataMessage, core::stick::{ Stick, StickId }};
use crate::index::StickAuthorityStore;
use crate::run::{ PgCommander, add_task, async_complete_task };
use crate::run::pgcommander::PgCommanderTaskSpec;
use crate::util::memoized::Memoized;
use crate::CountingPromise;
use crate::api::{ MessageSender, PeregrineCoreBase, AgentStore };

#[derive(Clone)]
pub struct StickStore {
    booted: CountingPromise,
    pending: Arc<Mutex<HashMap<StickId,Option<Result<Stick,DataMessage>>>>>,
    store: Memoized<StickId,Result<Arc<Stick>,DataMessage>>
}

impl StickStore {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> StickStore {
        let base = base.clone();
        let agent_store = agent_store.clone();
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let mut pending2 = pending.clone();
        StickStore {
            pending,
            booted: base.booted.clone(),
            store: Memoized::new(move |stick_id: &StickId, result| {
                pending2.lock().unwrap().insert(stick_id.clone(),None);
                let stick_id = stick_id.clone();
                let mut pending2 = pending2.clone();
                let agent_store = agent_store.clone();
                let handle = add_task(&base.commander,PgCommanderTaskSpec {
                    name: format!("stick-loader-{}",stick_id.get_id()),
                    prio: 3,
                    timeout: None,
                    slot: None,
                    task: Box::pin(async move {
                        // during this await, add should have been called
                        let value = match agent_store.stick_authority_store().await.try_lookup(&agent_store,stick_id.clone()).await {
                            Ok(()) => {
                                pending2.lock().unwrap().remove(&stick_id).flatten()
                                    .unwrap_or_else(|| Err(DataMessage::NoSuchStick(stick_id.clone())))
                                    .map(|x| Arc::new(x))
                            },
                            Err(v) => Err(v)
                        };
                        result.resolve(value);
                        Ok(())
                    })
                });
                async_complete_task(&base.commander,&base.messages,handle, |e| {
                    (DataMessage::StickAuthorityUnavailable(Box::new(e)),true)
                });
            })
        }
    }


    // TODO allow errors to be submitte
    pub fn add(&self, key: StickId, value: Stick) {
        self.pending.lock().unwrap().insert(key,Some(Ok(value)));
    }

    pub async fn get(&self, key: &StickId) -> Result<Arc<Stick>,DataMessage> {
        self.booted.wait().await;
        self.store.get(key).await.as_ref().clone()
    }
}
