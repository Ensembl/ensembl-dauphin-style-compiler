use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::run::{ PgDauphinTaskSpec };
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use crate::lane::ShapeRequest;
use crate::lane::programdata::ProgramData;
pub use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

async fn run(base: PeregrineCoreBase, agent_store: AgentStore, lane: ShapeRequest) -> Result<Arc<ShapeOutput>,DataMessage> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let shapes = ShapeOutput::new();
    payloads.insert("request".to_string(),Box::new(lane.clone()) as Box<dyn Any>);
    payloads.insert("out".to_string(),Box::new(shapes.clone()) as Box<dyn Any>);
    payloads.insert("data".to_string(),Box::new(ProgramData::new()) as Box<dyn Any>);
    base.dauphin.run_program(&agent_store.program_loader().await,PgDauphinTaskSpec {
        prio: 1,
        slot: None,
        timeout: None,
        program_name: lane.track().track().program_name().clone(),
        payloads: Some(payloads)
    }).await?;
    Ok(Arc::new(shapes))
}

#[derive(Clone)]
pub struct ShapeProgramRunAgent(Agent<ShapeRequest,Arc<ShapeOutput>>);

impl ShapeProgramRunAgent {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> ShapeProgramRunAgent {
        ShapeProgramRunAgent(Agent::new(MemoizedType::Cache(cache_size),"lane-run-cache",1,base,agent_store, run))
    }

    pub async fn get(&self, lane: &ShapeRequest) -> Result<Arc<ShapeOutput>,DataMessage> {
        self.0.get(lane).await.as_ref().clone()
    }
}
