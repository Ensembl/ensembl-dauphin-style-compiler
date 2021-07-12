use std::sync::{ Arc };
use crate::{DataMessage, core::stick::{ Stick, StickId }};
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn get_sticks(agent_store: &AgentStore, stick_id: &StickId) -> Result<Vec<Stick>,DataMessage> {
    agent_store.stick_authority_store().await.try_lookup(stick_id.clone()).await
}

async fn query_stick(agent_store: AgentStore, stick_cache: Memoized<StickId,Result<Arc<Stick>,DataMessage>>, stick_id: StickId) -> Result<Arc<Stick>,DataMessage> {
    let stick_cache = stick_cache.clone();
    let mut sticks = get_sticks(&agent_store,&stick_id).await?;
    let mut out = Err(DataMessage::NoSuchStick(stick_id.clone()));
    for stick in sticks.drain(..) {
        let stick = Arc::new(stick);
        if *stick.get_id() == stick_id {
            out = Ok(stick.clone());
        }
        stick_cache.warm(&stick.get_id().clone(),Ok(stick));
    }
    out
}

fn make_stick_cache(agent_store: &AgentStore) -> Memoized<StickId,Result<Arc<Stick>,DataMessage>> {
    //let pending = IndirectData::new();
    let agent_store = agent_store.clone();
    Memoized::new(MemoizedType::Store,move |stick_cache,stick_id: &StickId| {
        let agent_store = agent_store.clone();
        let stick_id = stick_id.clone();
        let stick_cache = stick_cache.clone();
        //let pending = pending.clone();
        Box::pin(async move { query_stick(agent_store.clone(),stick_cache.clone(),stick_id.clone()).await })
    })   
}

#[derive(Clone)]
pub struct StickStore(Memoized<StickId,Result<Arc<Stick>,DataMessage>>,PeregrineCoreBase);

impl StickStore {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> StickStore {
        StickStore(make_stick_cache(agent_store),base.clone())
    }

    pub async fn get(&self, key: &StickId) -> Result<Arc<Stick>,DataMessage> {
        self.1.booted.wait().await;
        self.0.get(key).await.as_ref().clone()
    }
}
