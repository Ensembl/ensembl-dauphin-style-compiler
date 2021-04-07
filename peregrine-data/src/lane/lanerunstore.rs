use std::any::Any;
use std::collections::HashMap;
use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::request::channel::{ Channel };
use crate::run::{ PgDauphinTaskSpec };
use crate::shape::ShapeOutput;
use crate::util::memoized::{ MemoizedType };
use super::lane::Lane;
use super::programdata::ProgramData;
pub use crate::util::message::DataMessage;
use crate::api::{ PeregrineCoreBase, AgentStore };

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct LaneRun {
    channel: Channel,
    program: String,
    lane: Lane
}

impl LaneRun {
    pub fn new(channel: Channel, program: &str, lane: &Lane) -> LaneRun {
        LaneRun {
            channel: channel.clone(),
            program: program.to_string(),
            lane: lane.clone()
        }
    }
}

#[derive(Clone)]
pub struct LaneRunOutput {
    shapes: ShapeOutput
}

impl LaneRunOutput {
    fn new() -> LaneRunOutput {
        LaneRunOutput {
            shapes: ShapeOutput::new()
        }
    }

    pub fn shapes(&self) -> &ShapeOutput { &self.shapes }
}

async fn run(base: PeregrineCoreBase, agent_store: AgentStore, lane_run: LaneRun) -> Result<Arc<LaneRunOutput>,DataMessage> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let pro = LaneRunOutput::new();
    payloads.insert("lane".to_string(),Box::new(lane_run.lane.clone()) as Box<dyn Any>);
    payloads.insert("out".to_string(),Box::new(pro.clone()) as Box<dyn Any>);
    payloads.insert("data".to_string(),Box::new(ProgramData::new()) as Box<dyn Any>);
    base.dauphin.run_program(&agent_store.program_loader().await,PgDauphinTaskSpec {
        prio: 1,
        slot: None,
        timeout: None,
        channel: lane_run.channel.clone(),
        program_name: lane_run.program.clone(),
        payloads: Some(payloads)
    }).await?;
    Ok(Arc::new(pro))
}

async fn program(base: PeregrineCoreBase, agent_store: AgentStore, lane: Lane) -> Result<LaneRun,DataMessage> {
    let stick_store = agent_store.stick_store().await;
    let lane_program_store = agent_store.lane_program_store().await;
    match lane.clone().build_lane_run(&stick_store,&lane_program_store).await {
        Ok(r) => Ok(r),
        Err(e) => {
            base.messages.send(e.clone());
            Err(DataMessage::DataMissing(Box::new(e)))
        }
    }
}

#[derive(Clone)]
pub struct LaneRunCache(Agent<LaneRun,Arc<LaneRunOutput>>);

impl LaneRunCache {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneRunCache {
        LaneRunCache(Agent::new(MemoizedType::Cache(cache_size),"lane-run-cache",1,base,agent_store, run))
    }

    pub async fn get(&self, lane_run: &LaneRun) -> Result<Arc<LaneRunOutput>,DataMessage> {
        self.0.get(lane_run).await.as_ref().clone()
    }
}

#[derive(Clone)]
pub struct LanePrograms(Agent<Lane,LaneRun>);

impl LanePrograms {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LanePrograms {
        LanePrograms(Agent::new(MemoizedType::Cache(cache_size),"lane-programs",1,base,agent_store, program))
    }

    pub async fn get(&self, lane: &Lane) -> Arc<Result<LaneRun,DataMessage>> {
        self.0.get(lane).await
    }
}

#[derive(Clone)]
pub struct LaneRunStore {
    store: LaneRunCache,
    programs: LanePrograms
}

impl LaneRunStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> LaneRunStore {
        LaneRunStore {
            store: LaneRunCache::new(cache_size,&base,&agent_store),
            programs: LanePrograms::new(cache_size,&base,&agent_store)
        }
    }
    
    pub async fn run(&self, lane: &Lane) -> Result<Arc<LaneRunOutput>,DataMessage> {
        match self.programs.get(&lane).await.as_ref() {
            Ok(lane_run) => {
                self.store.get(&lane_run).await
            },
            Err(e) => {
                Err(e.clone())
            }
        }
    }
}
