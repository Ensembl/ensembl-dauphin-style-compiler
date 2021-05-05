use std::sync::{ Arc };
use crate::{ agent::agent::Agent};
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use super::shaperequest::ShapeRequest;
use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn run(_base: PeregrineCoreBase, agent_store: AgentStore, shape_request: ShapeRequest) -> Result<ShapeOutput,DataMessage> {
    let better_shape_request = shape_request.better_request();
    match agent_store.shape_program_run_agent().await.get(&better_shape_request).await {
        Ok(pro) => {
            Ok(pro.filter(shape_request.region().min_value() as f64,shape_request.region().max_value() as f64))
        },
        Err(e) => {
            Err(DataMessage::DataMissing(Box::new(e.clone())))
        }
    }
}

#[derive(Clone)]
pub struct LaneStore(Agent<ShapeRequest,ShapeOutput>);

impl LaneStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneStore {
        LaneStore(Agent::new(MemoizedType::Cache(cache_size),"lane",1,base,agent_store, run))
    }

    pub async fn run(&self, lane: &ShapeRequest) -> Arc<Result<ShapeOutput,DataMessage>> {
        self.0.get(lane).await
    }
}
