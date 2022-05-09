use commander::cdr_current_time;
use peregrine_toolkit::log;
use std::collections::{HashMap, BTreeMap};
use std::sync::{ Arc };
use crate::api::{ PeregrineCoreBase };
use crate::core::channel::{Channel, PacketPriority};
use crate::request::messages::datares::DataRes;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::util::message::{ DataMessage };
use super::shaperequest::{ Region };

// TODO Memoized errors with retry semantics

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct DataRequest {
    region: Region,
    channel: Channel,
    name: String,
    scope: BTreeMap<String,Vec<String>>
}

impl DataRequest {
    pub fn new(channel: &Channel, name: &str, region: &Region) -> DataRequest {
        DataRequest {
            channel: channel.clone(),
            name: name.to_string(),
            region: region.clone(),
            scope: BTreeMap::new()
        }
    }

    pub fn add_scope(&self, key: &str, values: &[String]) -> DataRequest {
        let mut out = self.clone();
        out.scope.insert(key.to_string(),values.to_vec());
        out
    }
}

async fn run(base: PeregrineCoreBase, request: DataRequest, priority: PacketPriority) -> Result<Arc<DataRes>,DataMessage> {
    let backend = base.all_backends.backend(&request.channel);
    if !request.scope.is_empty() {
        log!("{:?}",request.scope);
    }
    backend.data(&request.name,&request.region,&priority).await.map(|x| Arc::new(x))
}

fn make_data_cache(cache_size: usize, base: &PeregrineCoreBase) -> Memoized<DataRequest,Result<Arc<DataRes>,DataMessage>> {
    let base = base.clone();
     Memoized::new(MemoizedType::Cache(cache_size),move |_,k: &DataRequest|{
        let base = base.clone();
        let k = k.clone();
        Box::pin(async move { run(base.clone(),k.clone(),PacketPriority::RealTime).await })
    })
}

#[derive(Clone)]
pub struct DataStore {
    cache: Memoized<DataRequest,Result<Arc<DataRes>,DataMessage>>,
    base: PeregrineCoreBase
}

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        DataStore { 
            base: base.clone(),
            cache: make_data_cache(cache_size,base)
        }
    }

    pub async fn get(&self, request: &DataRequest, priority: &PacketPriority) -> Result<(Arc<DataRes>,f64),DataMessage> {
        let start = cdr_current_time();
        let response = match priority {
            PacketPriority::RealTime => {
                self.cache.get(&request).await.as_ref().clone()
            },
            PacketPriority::Batch => {
                // XXX todo detect dups
                let data = run(self.base.clone(),request.clone(),PacketPriority::Batch).await;
                self.cache.warm(&request,data.clone());
                data
            }
        };
        let took_ms = cdr_current_time() - start;
        response.map(|r| (r,took_ms))
    }
}
