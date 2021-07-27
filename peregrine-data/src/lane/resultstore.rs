use std::collections::HashMap;
use crate::ProgramLoader;
use crate::util::builder::Builder;
use std::any::Any;
use std::sync::{ Arc };
use crate::shape::ShapeListBuilder;
use super::shaperequest::ShapeRequest;
use crate::util::message::DataMessage;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase, AgentStore };
use crate::run::{ PgDauphinTaskSpec };
use crate::lane::programdata::ProgramData;

async fn make_unfiltered_shapes(base: PeregrineCoreBase,program_loader: ProgramLoader, request: ShapeRequest) -> Result<Arc<ShapeListBuilder>,DataMessage> {
    base.booted.wait().await;
    let mut payloads = HashMap::new();
    let shapes = Builder::new(ShapeListBuilder::new());
    payloads.insert("request".to_string(),Box::new(request.clone()) as Box<dyn Any>);
    payloads.insert("out".to_string(),Box::new(shapes.clone()) as Box<dyn Any>);
    payloads.insert("data".to_string(),Box::new(ProgramData::new()) as Box<dyn Any>);
    payloads.insert("allotments".to_string(),Box::new(base.allotment_petitioner.clone()) as Box<dyn Any>);
    base.dauphin.run_program(&program_loader,PgDauphinTaskSpec {
        prio: 1,
        slot: None,
        timeout: None,
        program_name: request.track().track().program_name().clone(),
        payloads: Some(payloads)
    }).await?;
    Ok(Arc::new(shapes.build()))
}

fn make_unfiltered_cache(cache_size: usize, base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> Memoized<ShapeRequest,Result<Arc<ShapeListBuilder>,DataMessage>> {
    let base2 = base.clone();
    let program_loader = program_loader.clone();
    Memoized::new(MemoizedType::Cache(cache_size),move |_,request: &ShapeRequest| {
        let base = base2.clone();
        let program_loader = program_loader.clone(); 
        let request2 = request.clone();   
        Box::pin(async move {
            make_unfiltered_shapes(base,program_loader,request2.clone()).await
        })
    })
}

async fn make_filtered_shapes(unfiltered_shapes_cache: Memoized<ShapeRequest,Result<Arc<ShapeListBuilder>,DataMessage>>, shape_request: ShapeRequest) -> Result<Arc<ShapeListBuilder>,DataMessage> {
    let better_shape_request = shape_request.better_request();
    let unfiltered_shapes = unfiltered_shapes_cache.get(&better_shape_request).await;
    let unfiltered_shapes = unfiltered_shapes.as_ref().as_ref().map_err(|e| {
        DataMessage::DataMissing(Box::new(e.clone()))
    })?.clone();
    let region = shape_request.region();
    let filtered_shapes = unfiltered_shapes.filter(region.min_value() as f64,region.max_value() as f64);
    Ok(Arc::new(filtered_shapes))
}

fn make_filtered_cache(cache_size: usize, unfiltered_shapes_cache: Memoized<ShapeRequest,Result<Arc<ShapeListBuilder>,DataMessage>>) -> Memoized<ShapeRequest,Result<Arc<ShapeListBuilder>,DataMessage>> {
    let unfiltered_shapes_cache = unfiltered_shapes_cache.clone();
    Memoized::new(MemoizedType::Cache(cache_size),move |_,request: &ShapeRequest| {
        let unfiltered_shapes_cache = unfiltered_shapes_cache.clone();
        let request2 = request.clone();   
        Box::pin(async move {
            make_filtered_shapes(unfiltered_shapes_cache,request2.clone()).await
        })
    })
}

#[derive(Clone)]
pub struct LaneStore(Memoized<ShapeRequest,Result<Arc<ShapeListBuilder>,DataMessage>>);

impl LaneStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> LaneStore {
        // XXX both caches separate sizes
        let unfiltered_cache = make_unfiltered_cache(cache_size,base,program_loader);
        let filtered_cache = make_filtered_cache(32,unfiltered_cache);
        LaneStore(filtered_cache)
    }

    pub async fn run(&self, lane: &ShapeRequest) -> Arc<Result<Arc<ShapeListBuilder>,DataMessage>> {
        self.0.get(lane).await
    }
}
