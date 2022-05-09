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
