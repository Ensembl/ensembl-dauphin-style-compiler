use std::sync::{ Arc };
use crate::{ agent::agent::Agent};
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use super::lane::Lane;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn run(_base: PeregrineCoreBase, agent_store: AgentStore, wanted_lane: Lane) -> Result<ShapeOutput,DataMessage> {
    let lane_scaler = agent_store.lane_scaler().await;
    let scaled_lane = lane_scaler.get(&wanted_lane).await;
    let scaled_lane = scaled_lane.as_ref().as_ref().map_err(|e| e.clone())?;
    match agent_store.shape_program_run_agent().await.get(scaled_lane).await {
        Ok(pro) => {
            Ok(pro.filter(wanted_lane.min_value() as f64,wanted_lane.max_value() as f64))
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
