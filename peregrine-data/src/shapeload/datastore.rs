use commander::cdr_current_time;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::{lock};
use std::sync::{ Arc, Mutex };
use crate::PacketPriority;
use crate::api::{ PeregrineCoreBase };
use crate::request::minirequests::datareq::DataRequest;
use crate::request::minirequests::datares::{DataResponse};
use crate::util::lrucache::Cache;
use crate::util::memoized::{ Memoized, MemoizedType };

// TODO Memoized errors with retry semantics

#[cfg(debug_data_requests)]
fn debug_data_requests(request: &DataRequest){
    use peregrine_toolkit::debug_log;

    debug_log!("DataRequest for {:?}",request);
}

#[cfg(not(debug_data_requests))]
#[allow(unused)]
fn debug_data_requests(request: &DataRequest) {}

async fn run(base: PeregrineCoreBase, request: DataRequest, priority: PacketPriority) -> Result<DataResponse,Error> {
    debug_data_requests(&request);
    let backend = base.all_backends.backend(request.channel())?;
    backend.data(&request,&priority).await
}

fn make_data_cache(cache_size: usize, base: &PeregrineCoreBase, prio: PacketPriority) -> Memoized<DataRequest,Result<DataResponse,Error>> {
    let base = base.clone();
    Memoized::new(MemoizedType::Cache(cache_size),move |_,k: &DataRequest|{
        let base = base.clone();
        let prio = prio.clone();
        let k = k.clone();
        Box::pin(async move { run(base.clone(),k.clone(),prio).await })
    })
}

const STAT_WINDOW_SIZE : usize = 20;

#[cfg(debug_canvasstore)]
struct Stats {
    source: Vec<bool>,
}

#[cfg(debug_canvasstore)]
impl Stats {
    fn new() -> Stats { Stats { source: vec![] } }


    fn add(&mut self, priority: &PacketPriority) { 
        use peregrine_toolkit::log;

        self.source.push(priority.is_high());
        if self.source.len() >= STAT_WINDOW_SIZE {
            let perc = self.source.drain(..).filter(|x| *x).count() * 100 / STAT_WINDOW_SIZE;
            log!("{}% of requests originally from lo preloading",perc);
        }
    }
}

#[cfg(not(debug_canvasstore))]
struct Stats;

#[cfg(not(debug_canvasstore))]
impl Stats {
    fn new() -> Stats { Stats }
    fn add(&mut self, _priority: &PacketPriority) {}
}


#[derive(Clone)]
pub struct DataStore {
    invariant_cache: Arc<Mutex<Cache<DataRequest,Result<DataResponse,Error>>>>,
    cache: Memoized<DataRequest,Result<DataResponse,Error>>,
    batch_cache: Memoized<DataRequest,Result<DataResponse,Error>>,
    stats: Arc<Mutex<Stats>>
}

impl DataStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> DataStore {
        DataStore { 
            invariant_cache: Arc::new(Mutex::new(Cache::new(cache_size))),
            cache: make_data_cache(cache_size,base,PacketPriority::RealTime),
            batch_cache: make_data_cache(cache_size,base,PacketPriority::Batch),
            stats: Arc::new(Mutex::new(Stats::new()))
        }
    }

    pub async fn get(&self, request: &DataRequest, priority: &PacketPriority) -> Result<(DataResponse,f64),Error> {
        let start = cdr_current_time();
        /* maybe there's an invariant version? */
        if let Some(response) = lock!(self.invariant_cache).get(&request.to_invariant()).cloned().transpose()? {
            let took_ms = cdr_current_time() - start;
            return Ok((response,took_ms));
        }
        /* needs full lookup or new request */
        let response = match priority {
            PacketPriority::RealTime => {
                self.cache.get(&request).await.as_ref().clone()
            },
            PacketPriority::Batch => {
                if let Some(value) = self.cache.try_underway(&request).await {
                    /* Already in main cache, don't pull again */
                    value.as_ref().clone()
                } else {
                    let data = self.batch_cache.get(&request).await;
                    self.cache.warm(&request,data.as_ref().clone());
                    data.as_ref().clone()
                }
            }
        };
        if let Ok(response) = &response{
            if response.is_invariant() {
                lock!(self.invariant_cache).put(&request.to_invariant(),Ok(response.clone()));
            }
            if priority.is_high() {
                lock!(self.stats).add(&response.original_priority());
            }
        }
        let took_ms = cdr_current_time() - start;
        response.map(|r| (r,took_ms))
    }
}
