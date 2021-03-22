use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::{DataMessage, core::stick::{ Stick, StickId }};
use crate::util::memoized::{ MemoizedType };
use crate::util::indirectanswer::IndirectData;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn get_stick(agent_store: AgentStore, pending: IndirectData<StickId,Result<Stick,DataMessage>>, stick_id: StickId) -> Result<Arc<Stick>,DataMessage> {
    let p = pending.get_future(&stick_id);
    if let Err(e) = agent_store.stick_authority_store().await.try_lookup(stick_id.clone()).await {
        pending.set(&stick_id,Err(e));
    }
    pending.set(&stick_id,Err(DataMessage::NoSuchStick(stick_id.clone())));
    p.await.map(|r| Arc::new(r))
}

#[derive(Clone)]
pub struct StickStore {
    agent: Agent<StickId,Arc<Stick>>,
    pending: IndirectData<StickId,Result<Stick,DataMessage>>,
    base: PeregrineCoreBase
}

impl StickStore {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> StickStore {
        let indirect_data = IndirectData::new_pending();
        let indirect_data2 = indirect_data.clone();
        let cb = move |_base,agent_store,stick_id| {
            get_stick(agent_store,indirect_data2.clone(),stick_id)
        };
        let agent = Agent::new(MemoizedType::Store,"stick-loader",3,base,agent_store,
        cb);
        StickStore {
            agent,
            pending: indirect_data,
            base: base.clone()
        }
    }

    pub fn add(&self, key: StickId, value: Stick) {
        self.pending.set(&key,Ok(value));
    }

    pub async fn get(&self, key: &StickId) -> Result<Arc<Stick>,DataMessage> {
        self.base.booted.wait().await;
        self.agent.get(key).await.as_ref().clone()
    }
}
