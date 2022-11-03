use std::sync::Mutex;
use commander::cdr_current_time;
use peregrine_toolkit::error::Error;
use std::collections::HashMap;
use crate::{ProgramShapesBuilder, ObjectBuilder };
use std::any::Any;
use std::sync::{ Arc };
use crate::shape::{AbstractShapesContainer};
use super::programloader::ProgramLoader;
use super::loadshapes::LoadMode;
use super::shaperequest::ShapeRequest;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };
use crate::run::{ PgDauphinTaskSpec };
use peregrine_toolkit::{lock};

async fn make_unfiltered_shapes(base: PeregrineCoreBase, program_loader: ProgramLoader, request: ShapeRequest, mode: LoadMode) -> Result<Arc<AbstractShapesContainer>,Error> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let shapes = Arc::new(Mutex::new(Some(ProgramShapesBuilder::new(&lock!(base.assets).clone()))));
    let net_ms = Arc::new(Mutex::new(0.));
    /* This is what is being requested */
    payloads.insert("request".to_string(),Box::new(request.clone()) as Box<dyn Any>);
    /* This is where the output goes */
    payloads.insert("out".to_string(),Box::new(shapes.clone()) as Box<dyn Any>);
    /* Temporary instances of types needed by scripts */
    payloads.insert("builder".to_string(),Box::new(ObjectBuilder::new()) as Box<dyn Any>);
    payloads.insert("net_time".to_string(),Box::new(net_ms.clone()) as Box<dyn Any>);
    payloads.insert("mode".to_string(),Box::new(mode.clone()) as Box<dyn Any>);
    let start = cdr_current_time();
    base.dauphin.run_program(&program_loader,&base.channel_registry,PgDauphinTaskSpec {
        prio: if mode.high_priority() { 2 } else { 9 },
        program_name: request.track().track().program_name().clone(),
        payloads: Some(payloads)
    }).await.map_err(|e| Error::operr(&format!("dauphin program failed: {:?}",e)))?;
    let took_ms = cdr_current_time() - start;
    let net_time_ms = *net_ms.lock().unwrap();
    base.metrics.program_run(&request.track().track().program_name().indicative_name(),request.region().scale().get_index(),!mode.build_shapes(),net_time_ms,took_ms);
    let shapes = lock!(shapes).take().unwrap().to_abstract_shapes_container();
    Ok(Arc::new(shapes))
}

fn make_unfiltered_cache(kind: MemoizedType, base: &PeregrineCoreBase, program_loader: &ProgramLoader, mode: LoadMode) -> Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>> {
    let base2 = base.clone();
    let program_loader = program_loader.clone();
    let mode = mode.clone();
    Memoized::new(kind,move |_,request: &ShapeRequest| {
        let base = base2.clone();
        let program_loader = program_loader.clone(); 
        let request2 = request.clone();   
        let mode = mode.clone();
        Box::pin(async move {
            make_unfiltered_shapes(base,program_loader,request2.clone(),mode.clone()).await
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
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> ShapeStore {
        // XXX both caches separate sizes
        let unfiltered_cache = make_unfiltered_cache(MemoizedType::Cache(cache_size),base,program_loader,LoadMode::RealTime);
        let filtered_cache = make_filtered_cache(MemoizedType::Cache(cache_size),unfiltered_cache);
        let batch_unfiltered_cache = make_unfiltered_cache(MemoizedType::None,base,program_loader,LoadMode::Batch);
        let batch_filtered_cache = make_filtered_cache(MemoizedType::None,batch_unfiltered_cache);
        let network_unfiltered_cache = make_unfiltered_cache(MemoizedType::None,base,program_loader,LoadMode::Network);
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
