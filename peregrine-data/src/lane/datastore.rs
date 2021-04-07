use std::sync::{ Arc };
use crate::agent::agent::Agent;
use crate::api::{ PeregrineCoreBase, AgentStore };
use crate::util::memoized::{MemoizedType };
use crate::util::message::{ DataMessage };
use super::lane::Lane;
use crate::{ Channel };
use crate::request::data::{ DataCommandRequest, DataResponse };

// TODO Memoized errors with retry semantics

async fn run(base: PeregrineCoreBase,_agent_store: AgentStore, (lane,channel,name): (Lane,Channel,String)) -> Result<Arc<Box<DataResponse>>,DataMessage> {
    DataCommandRequest::new(&channel,&name,&lane).execute(base.manager).await.map(|x| Arc::new(x))
}

#[derive(Clone)]
pub struct DataStore(Agent<(Lane,Channel,String),Arc<Box<DataResponse>>>);

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, agent_store: &AgentStore) -> DataStore {
        DataStore(Agent::new(MemoizedType::Cache(cache_size),"data",1,base,agent_store, run))
    }

    pub async fn get(&self, lane: &Lane, channel: &Channel, name: &str) -> Result<Arc<Box<DataResponse>>,DataMessage> {
        self.0.get(&(lane.clone(),channel.clone(),name.to_string())).await.as_ref().clone()
    }
}
