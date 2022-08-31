use commander::cdr_current_time;
use std::sync::{ Arc };
use crate::api::{ PeregrineCoreBase };
use crate::core::channel::{PacketPriority};
use crate::request::messages::datareq::DataRequest;
use crate::request::messages::datares::DataRes;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::util::message::{ DataMessage };

// TODO Memoized errors with retry semantics

async fn run(base: PeregrineCoreBase, request: DataRequest, priority: PacketPriority) -> Result<Arc<DataRes>,DataMessage> {
    let backend = base.all_backends.backend(request.channel());
    backend.data(&request,&priority).await.map(|x| Arc::new(x))
}

fn make_data_cache(cache_size: usize, base: &PeregrineCoreBase, prio: PacketPriority) -> Memoized<DataRequest,Result<Arc<DataRes>,DataMessage>> {
    let base = base.clone();
     Memoized::new(MemoizedType::Cache(cache_size),move |_,k: &DataRequest|{
        let base = base.clone();
        let prio = prio.clone();
        let k = k.clone();
        Box::pin(async move { run(base.clone(),k.clone(),prio).await })
    })
}

#[derive(Clone)]
pub struct DataStore {
    cache: Memoized<DataRequest,Result<Arc<DataRes>,DataMessage>>,
    batch_cache: Memoized<DataRequest,Result<Arc<DataRes>,DataMessage>>
}

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        DataStore { 
            cache: make_data_cache(cache_size,base,PacketPriority::RealTime),
            batch_cache: make_data_cache(cache_size,base,PacketPriority::Batch)
        }
    }

    pub async fn get(&self, request: &DataRequest, priority: &PacketPriority) -> Result<(Arc<DataRes>,f64),DataMessage> {
        let start = cdr_current_time();
        let response = match priority {
            PacketPriority::RealTime => {
                self.cache.get(&request).await.as_ref().clone()
            },
            PacketPriority::Batch => {
                let data = self.batch_cache.get(&request).await;
                self.cache.warm(&request,data.as_ref().clone());
                data.as_ref().clone()
            }
        };
        let took_ms = cdr_current_time() - start;
        response.map(|r| (r,took_ms))
    }
}
