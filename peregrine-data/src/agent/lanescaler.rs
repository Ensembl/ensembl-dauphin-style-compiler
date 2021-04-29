use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::util::memoized::{ MemoizedType };
use crate::lane::Lane;
pub use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn scale(base: PeregrineCoreBase, agent_store: AgentStore, lane: Lane) -> Result<Lane,DataMessage> {
    let stick_store = agent_store.stick_store().await;
    let lane_program_lookup = agent_store.lane_program_lookup().await;
    match lane.clone().scaler(&stick_store,&lane_program_lookup).await {
        Ok(r) => Ok(r),
        Err(e) => {
            base.messages.send(e.clone());
            Err(DataMessage::DataMissing(Box::new(e)))
        }
    }
}

#[derive(Clone)]
pub struct LaneScaler(Agent<Lane,Lane>);

impl LaneScaler {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneScaler {
        LaneScaler(Agent::new(MemoizedType::Cache(cache_size),"lane-programs",1,base,agent_store, scale))
    }

    pub async fn get(&self, lane: &Lane) -> Arc<Result<Lane,DataMessage>> {
        self.0.get(lane).await
    }
}
