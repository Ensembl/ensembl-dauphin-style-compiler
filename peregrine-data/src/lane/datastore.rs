use std::sync::{ Arc };
use crate::api::{ PeregrineCoreBase };
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::util::message::{ DataMessage };
use super::shaperequest::{ Region };
use crate::{Channel, PacketPriority};
use crate::request::data::{ DataCommandRequest, DataResponse };

// TODO Memoized errors with retry semantics

async fn run(base: PeregrineCoreBase, (region,channel,name): (Region,Channel,String), priority: PacketPriority) -> Result<Arc<Box<DataResponse>>,DataMessage> {
    DataCommandRequest::new(&channel,&name,&region).execute(base.manager,&priority).await.map(|x| Arc::new(x))
}

fn make_data_cache(cache_size: usize, base: &PeregrineCoreBase) -> Memoized<(Region,Channel,String),Result<Arc<Box<DataResponse>>,DataMessage>> {
    let base = base.clone();
     Memoized::new(MemoizedType::Cache(cache_size),move |_,k: &(Region,Channel,String)|{
        let base = base.clone();
        let k = k.clone();
        Box::pin(async move { run(base.clone(),k.clone(),PacketPriority::RealTime).await })
    })
}

#[derive(Clone)]
pub struct DataStore {
    cache: Memoized<(Region,Channel,String),Result<Arc<Box<DataResponse>>,DataMessage>>,
    base: PeregrineCoreBase
}

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        DataStore { 
            base: base.clone(),
            cache: make_data_cache(cache_size,base)
        }
    }

    pub async fn get(&self, region: &Region, channel: &Channel, name: &str, priority: &PacketPriority) -> Result<Arc<Box<DataResponse>>,DataMessage> {
        let location = (region.clone(),channel.clone(),name.to_string());
        match priority {
            PacketPriority::RealTime => {
                self.cache.get(&location).await.as_ref().clone()
            },
            PacketPriority::Batch => {
                // XXX todo detect dups
                let data = run(self.base.clone(),location.clone(),PacketPriority::Batch).await;
                self.cache.warm(&location,data.clone());
                data
            }
        }
    }
}
