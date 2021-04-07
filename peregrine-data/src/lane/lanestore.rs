use std::sync::{ Arc };
use crate::{ agent::agent::Agent};
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use super::lane::Lane;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn run(_base: PeregrineCoreBase,agent_store: AgentStore, lane: Lane) -> Result<ShapeOutput,DataMessage> {
    match agent_store.lane_run_store().await.run(&lane).await {
        Ok(pro) => {
            Ok(pro.shapes().filter(lane.min_value() as f64,lane.max_value() as f64))
        },
        Err(e) => {
            Err(DataMessage::DataMissing(Box::new(e.clone())))
        }
    }
}

#[derive(Clone)]
pub struct LaneStore(Agent<Lane,ShapeOutput>);

impl LaneStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneStore {
        LaneStore(Agent::new(MemoizedType::Cache(cache_size),"lane",1,base,agent_store, run))
    }

    pub async fn run(&self, lane: &Lane) -> Arc<Result<ShapeOutput,DataMessage>> {
        self.0.get(lane).await
    }
}
