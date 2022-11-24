use std::sync::Mutex;
use commander::cdr_current_time;
use peregrine_toolkit::error::Error;
use std::collections::HashMap;
use crate::run::pgdauphin::PgDauphinTaskSpec;
use crate::shape::originstats::OriginStats;
use crate::{ProgramShapesBuilder, ObjectBuilder };
use std::any::Any;
use std::sync::{ Arc };
use crate::shape::{AbstractShapesContainer};
use super::loadshapes::LoadMode;
use super::shaperequest::ShapeRequest;
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };
use peregrine_toolkit::lock;

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

fn add_payloads(payloads: &mut HashMap<String,Box<dyn Any>>,
        request: &ShapeRequest, mode: &LoadMode, run_report: &Arc<Mutex<RunReport>>, 
        shapes: &Arc<Mutex<Option<ProgramShapesBuilder>>>) {
    /* This is the region requested */
    payloads.insert("request".to_string(),Box::new(request.clone()) as Box<dyn Any>);

    /* This is where the output goes */
    payloads.insert("out".to_string(),Box::new(shapes.clone()) as Box<dyn Any>);

    /* Temporary instances of types needed by scripts */
    payloads.insert("builder".to_string(),Box::new(ObjectBuilder::new()) as Box<dyn Any>);

    /* A report about resources consumed by script */
    payloads.insert("report".to_string(),Box::new(run_report.clone()) as Box<dyn Any>);

    /* Context of request (eg priority) */
    payloads.insert("mode".to_string(),Box::new(mode.clone()) as Box<dyn Any>);
}

async fn make_unfiltered_shapes(base: PeregrineCoreBase, request: ShapeRequest, mode: LoadMode, will_discard_output: bool) -> Result<Arc<AbstractShapesContainer>,Error> {
    base.booted.wait().await;
    let shapes = Arc::new(Mutex::new(Some(ProgramShapesBuilder::new(&lock!(base.assets).clone(),&mode))));
    let mut payloads = HashMap::new();
    let run_report = Arc::new(Mutex::new(RunReport::new()));
    add_payloads(&mut payloads,&request,&mode,&run_report,&shapes);
    let start = cdr_current_time();
    base.dauphin.run_program(&base.channel_registry,PgDauphinTaskSpec {
        program: request.track().track().program().clone(),
        mapping: request.track().track().mapping().clone(),
        track_base: request.track().track().track_base().clone(),
        payloads: Some(payloads)
    },&mode).await.map_err(|e| e.context(&format!("running {}",request.track().track().program().name().indicative_name())))?;
    let took_ms = cdr_current_time() - start;
    let net_time_ms = lock!(run_report).net_ms;
    if will_discard_output {
        return Ok(Arc::new(AbstractShapesContainer::empty(&mode)))
    }
    base.metrics.program_run(&request.track().track().program().name().indicative_name(),request.region().scale().get_index(),!mode.build_shapes(),net_time_ms,took_ms);
    let shapes = lock!(shapes).take().unwrap().to_abstract_shapes_container();
    Ok(Arc::new(shapes))
}

fn make_unfiltered_cache(kind: MemoizedType, base: &PeregrineCoreBase, mode: LoadMode, will_discard_output: bool) -> Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>> {
    let base2 = base.clone();
    let mode = mode.clone();
    Memoized::new(kind,move |_,request: &ShapeRequest| {
        let base = base2.clone();
        let request2 = request.clone();   
        let mode = mode.clone();
        Box::pin(async move {
            make_unfiltered_shapes(base,request2.clone(),mode.clone(),will_discard_output).await
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
    network: Memoized<ShapeRequest,Result<Arc<AbstractShapesContainer>,Error>>,
    stats: Arc<Mutex<OriginStats>>
}

impl ShapeStore {
    pub fn new(cache_size: usize, base: &PeregrineCoreBase) -> ShapeStore {
        // XXX both caches separate sizes
        let unfiltered_cache = make_unfiltered_cache(MemoizedType::Cache(cache_size),base,LoadMode::RealTime,false);
        let filtered_cache = make_filtered_cache(MemoizedType::Cache(cache_size),unfiltered_cache);
        let batch_unfiltered_cache = make_unfiltered_cache(MemoizedType::Cache(cache_size),base,LoadMode::Batch,false);
        let batch_filtered_cache = make_filtered_cache(MemoizedType::None,batch_unfiltered_cache);
        let network_unfiltered_cache = make_unfiltered_cache(MemoizedType::Cache(cache_size),base,LoadMode::Network,true);
        let network_filtered_cache = make_filtered_cache(MemoizedType::None,network_unfiltered_cache);
        ShapeStore {
            realtime: filtered_cache,
            batch: batch_filtered_cache,
            network: network_filtered_cache,
            stats: Arc::new(Mutex::new(OriginStats::empty()))
        }
    }

    pub async fn run(&self, lane: &ShapeRequest, mode: &LoadMode) -> Arc<Result<Arc<AbstractShapesContainer>,Error>> {
        let shapes = match mode {
            /* really get the shapes, we really want them, NOW! */
            LoadMode::RealTime => {
                self.realtime.get(lane).await
            },
            /* really get the shapes, we may need them in the future */
            LoadMode::Batch => {
                if let Some(value) = self.realtime.try_get(lane) {
                    value
                } else {
                    let value = self.batch.get(lane).await;
                    self.realtime.warm(lane,value.as_ref().clone());
                    value
                }
            },
            /* run the network requests, don't worry about the output */
            LoadMode::Network => {
                let value = self.realtime.try_get(lane)
                    .or_else(|| self.batch.try_get(lane))
                    .or_else(|| self.network.try_get(lane));
                match value {
                    Some(x) => x,
                    None => self.network.get(lane).await
                }
            }
        };
        if let LoadMode::RealTime = mode {
            /* genuine request, so include in stats */
            if let Ok(shapes) = shapes.as_ref() {
                let mut stats = lock!(self.stats);
                stats.merge(shapes.stats());
                stats.report();
            }
        }
        shapes
    }
}
