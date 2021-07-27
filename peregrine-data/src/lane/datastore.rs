use std::sync::{ Arc };
use crate::api::{ PeregrineCoreBase, AgentStore };
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::util::message::{ DataMessage };
use super::shaperequest::{ Region };
use crate::{ Channel };
use crate::request::data::{ DataCommandRequest, DataResponse };

// TODO Memoized errors with retry semantics

async fn run(base: PeregrineCoreBase, (region,channel,name): (Region,Channel,String)) -> Result<Arc<Box<DataResponse>>,DataMessage> {
    DataCommandRequest::new(&channel,&name,&region).execute(base.manager).await.map(|x| Arc::new(x))
}

fn make_data_cache(cache_size: usize, base: &PeregrineCoreBase) -> Memoized<(Region,Channel,String),Result<Arc<Box<DataResponse>>,DataMessage>> {
    let base = base.clone();
     Memoized::new(MemoizedType::Cache(cache_size),move |_,k: &(Region,Channel,String)|{
        let base = base.clone();
        let k = k.clone();
        Box::pin(async move { run(base.clone(),k.clone()).await })
    })
}

#[derive(Clone)]
pub struct DataStore(Memoized<(Region,Channel,String),Result<Arc<Box<DataResponse>>,DataMessage>>);

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        DataStore(make_data_cache(cache_size,base))
    }

    pub async fn get(&self, region: &Region, channel: &Channel, name: &str) -> Result<Arc<Box<DataResponse>>,DataMessage> {
        self.0.get(&(region.clone(),channel.clone(),name.to_string())).await.as_ref().clone()
    }
}
