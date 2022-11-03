use std::sync::Mutex;
use commander::cdr_current_time;
use peregrine_toolkit::error::Error;
use std::collections::HashMap;
use crate::{ProgramShapesBuilder, ObjectBuilder };
use std::any::Any;
use std::sync::{ Arc };
use crate::shape::{AbstractShapesContainer};
use super::loadshapes::LoadMode;
use super::shaperequest::ShapeRequest;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };
use crate::run::{ PgDauphinTaskSpec };
use peregrine_toolkit::{lock};

pub struct RunReport {
    pub net_ms: f64
}

impl RunReport {
    fn new() -> RunReport {
        RunReport {
            net_ms: 0.
        }
    }
}

async fn make_unfiltered_shapes(base: PeregrineCoreBase, request: ShapeRequest, mode: LoadMode) -> Result<Arc<AbstractShapesContainer>,Error> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let shapes = Arc::new(Mutex::new(Some(ProgramShapesBuilder::new(&lock!(base.assets).clone()))));
    let run_report = Arc::new(Mutex::new(RunReport::new()));
    /* This is what is being requested */
    payloads.insert("request".to_string(),Box::new(request.clone()) as Box<dyn Any>);
    /* This is where the output goes */
    payloads.insert("out".to_string(),Box::new(shapes.clone()) as Box<dyn Any>);
    /* Temporary instances of types needed by scripts */
    payloads.insert("builder".to_string(),Box::new(ObjectBuilder::new()) as Box<dyn Any>);
    /* A report about resources consumed by script */
    payloads.insert("report".to_string(),Box::new(run_report.clone()) as Box<dyn Any>);
    /* Context of request (eg priority) */
    payloads.insert("mode".to_string(),Box::new(mode.clone()) as Box<dyn Any>);
    let start = cdr_current_time();
    base.dauphin.run_program(&base.channel_registry,PgDauphinTaskSpec {
        program_name: request.track().track().program().clone(),
        payloads: Some(payloads)
    },&mode).await.map_err(|e| Error::operr(&format!("dauphin program failed: {:?}",e)))?;
    let took_ms = cdr_current_time() - start;
    let net_time_ms = lock!(run_report).net_ms;
    base.metrics.program_run(&request.track().track().program().indicative_name(),request.region().scale().get_index(),!mode.build_shapes(),net_time_ms,took_ms);
    let shapes = lock!(shapes).take().unwrap().to_abstract_shapes_container();
    Ok(Arc::new(shapes))
}

fn make_unfiltered_cache(kind: MemoizedType, base: &PeregrineCoreBase, mode: LoadMode) -> Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>> {
    let base2 = base.clone();
    let mode = mode.clone();
    Memoized::new(kind,move |_,request: &ShapeRequest| {
        let base = base2.clone();
        let request2 = request.clone();   
        let mode = mode.clone();
        Box::pin(async move {
            make_unfiltered_shapes(base,request2.clone(),mode.clone()).await
        })
    })
}

async fn make_filtered_shapes(unfiltered_shapes_cache: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>, shape_request: ShapeRequest) -> Result<Arc<AbstractShapesContainer>,Error> {
    let better_shape_request = shape_request.better_request();
    let unfiltered_shapes = unfiltered_shapes_cache.get(&better_shape_request).await;
    let unfiltered_shapes = unfiltered_shapes.as_ref().clone()?;
    let region = shape_request.region();
    let filtered_shapes = unfiltered_shapes.filter(region.min_value() as f64,region.max_value() as f64);
    Ok(Arc::new(filtered_shapes))
}

fn make_filtered_cache(kind: MemoizedType, unfiltered_shapes_cache: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>) -> Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>> {
    let unfiltered_shapes_cache = unfiltered_shapes_cache.clone();
    Memoized::new(kind,move |_,request: &ShapeRequest| {
        let unfiltered_shapes_cache = unfiltered_shapes_cache.clone();
        let request2 = request.clone();   
        Box::pin(async move {
            make_filtered_shapes(unfiltered_shapes_cache,request2.clone()).await
        })
    })
}

#[derive(Clone)]
pub struct ShapeStore {
    realtime: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>,
    batch: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>,
    network: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>
}

impl ShapeStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> ShapeStore {
        // XXX both caches separate sizes
        let unfiltered_cache = make_unfiltered_cache(MemoizedType::Cache(cache_size),base,LoadMode::RealTime);
        let filtered_cache = make_filtered_cache(MemoizedType::Cache(cache_size),unfiltered_cache);
        let batch_unfiltered_cache = make_unfiltered_cache(MemoizedType::None,base,LoadMode::Batch);
        let batch_filtered_cache = make_filtered_cache(MemoizedType::None,batch_unfiltered_cache);
        let network_unfiltered_cache = make_unfiltered_cache(MemoizedType::None,base,LoadMode::Network);
        let network_filtered_cache = make_filtered_cache(MemoizedType::None,network_unfiltered_cache);
        ShapeStore {
            realtime: filtered_cache,
            batch: batch_filtered_cache,
            network: network_filtered_cache,
        }
    }

    pub async fn run(&self, lane: &ShapeRequest, mode: &LoadMode) -> Arc<Result<Arc<AbstractShapesContainer>,Error>> {
        match mode {
            LoadMode::RealTime => {
                self.realtime.get(lane).await
            },
            LoadMode::Batch => {
                if let Some(value) = self.realtime.try_get(lane) {
                    value
                } else {
                    let value = self.batch.get(lane).await;
                    self.realtime.warm(lane,value.as_ref().clone());
                    value
                }
            },
            LoadMode::Network => {
                if let Some(value) = self.realtime.try_get(lane) {
                    value
                } else if let Some(value) = self.batch.try_get(lane) {
                    value    
                } else if let Some(value) = self.network.try_get(lane) {
                    value
                } else {
                    self.network.get(lane).await
                }
            }
        }
    }
}
