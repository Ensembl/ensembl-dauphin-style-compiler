use std::sync::{ Arc };
use peregrine_toolkit::error::Error;

use crate::{ core::{stick::{ Stick, StickId }, channel::channelregistry::ChannelRegistry}, AllBackends};
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };

async fn get_sticks(all_backends: &AllBackends, channel_registry: &ChannelRegistry, stick_id: &StickId) -> Result<Vec<Stick>,Error> {
    let mut sticks = vec![];
    for backend_namespace in &channel_registry.all() {
        let backend = all_backends.backend(backend_namespace)?;
        let stick = backend.stick(stick_id).await?;
        if let Some(stick) = stick {
            sticks.push(stick);
        }
    }
    Ok(sticks)
}

async fn query_stick(all_backends: &AllBackends, channel_registry: &ChannelRegistry, stick_cache: Memoized<StickId,Result<Arc<Stick>,Error>>, stick_id: StickId) -> Result<Arc<Stick>,Error> {
    let stick_cache = stick_cache.clone();
    let mut sticks = get_sticks(all_backends,channel_registry,&stick_id).await?;
    let mut out = Err(Error::operr(&format!("no such stick: {}",stick_id.get_id())));
    for stick in sticks.drain(..) {
        let stick = Arc::new(stick);
        if *stick.get_id() == stick_id {
            out = Ok(stick.clone());
        }
        stick_cache.warm(&stick.get_id().clone(),Ok(stick));
    }
    out
}

fn make_stick_cache(all_backends: &AllBackends, channel_registry: &ChannelRegistry) -> Memoized<StickId,Result<Arc<Stick>,Error>> {
    let all_backends = all_backends.clone();
    let channel_registry = channel_registry.clone();
    Memoized::new(MemoizedType::Store,move |stick_cache,stick_id: &StickId| {
        let all_backends = all_backends.clone();
        let channel_registry = channel_registry.clone();    
        let stick_id = stick_id.clone();
        let stick_cache = stick_cache.clone();
        Box::pin(async move { query_stick(&all_backends,&channel_registry,stick_cache.clone(),stick_id.clone()).await })
    })   
}

#[derive(Clone)]
pub struct StickStore {
    sticks: Memoized<StickId,Result<Arc<Stick>,Error>>,
    base: PeregrineCoreBase
}

impl StickStore {
    pub fn new(base: &PeregrineCoreBase) -> StickStore {
        StickStore {
            sticks: make_stick_cache(&base.all_backends,&base.channel_registry),
            base: base.clone()
        }
    }

    pub async fn get(&self, key: &StickId) -> Result<Arc<Stick>,Error> {
        self.base.booted.wait().await;
        self.sticks.get(key).await.as_ref().clone()
    }
}
