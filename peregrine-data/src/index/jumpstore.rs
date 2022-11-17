use std::sync::{ Arc };
use commander::PromiseFuture;
use peregrine_toolkit::error::Error;

use crate::{PgCommanderTaskSpec, add_task, async_complete_task, core::{stick::{ StickId }, channel::channelregistry::ChannelRegistry}, AllBackends};
use crate::util::memoized::{ Memoized, MemoizedType };
use crate::api::{ PeregrineCoreBase };

async fn get_jump(all_backends: &AllBackends, channel_registry: &ChannelRegistry, location: &str) -> Result<Option<(String,u64,u64)>,Error> {
    for backend_namespace in &channel_registry.all() {
        let backend = all_backends.backend(backend_namespace)?;
        if let Some(location) = backend.jump(location).await? {
            return Ok(Some((location.stick.to_string(),location.left,location.right)))
        }
    }
    Ok(None)
}

async fn query_jump(all_backends: &AllBackends, channel_registry: &ChannelRegistry, location: &str) -> Result<Arc<(String,u64,u64)>,Error> {
    let jump = get_jump(all_backends,channel_registry,location).await?.ok_or_else(||
        Error::operr(&format!("no such jump: {}",location))
    )?;
    Ok(Arc::new(jump))
}

fn make_jump_cache(all_backends: &AllBackends, channel_registry: &ChannelRegistry) -> Memoized<String,Result<Arc<(String,u64,u64)>,Error>> {
    let all_backends = all_backends.clone();
    let channel_registry = channel_registry.clone();
    Memoized::new(MemoizedType::Cache(128),move |_,location: &String| {
        let all_backends = all_backends.clone();
        let channel_registry = channel_registry.clone();
        let location = location.clone();
        Box::pin(async move { query_jump(&all_backends,&channel_registry,&location).await })
    })   
}

#[derive(Clone)]
pub struct JumpStore(Memoized<String,Result<Arc<(String,u64,u64)>,Error>>,PeregrineCoreBase);

impl JumpStore {
    pub fn new(base: &PeregrineCoreBase) -> JumpStore {
        JumpStore(make_jump_cache(&base.all_backends,&base.channel_registry),base.clone())
    }

    pub async fn get(&self, location: &String) -> Result<Arc<(String,u64,u64)>,Error> {
        self.1.booted.wait().await;
        self.0.get(location).await.as_ref().clone()
    }

    pub(crate) fn jump(&self, location: &str, promise: PromiseFuture<Option<(StickId,f64,f64)>>) {
        let self2 = self.clone();
        let location = location.to_string();
        let handle = add_task(&self.1.commander,PgCommanderTaskSpec {
            name: "jump".to_string(),
            prio: 4,
            timeout: None,
            slot: None,
            task: Box::pin(async move {
                let result = match self2.get(&location).await {
                    Ok(result) => {
                        let (stick,left,right) = result.as_ref();
                        let left = *left as f64;
                        let right = *right as f64;
                        Some((StickId::new(&stick),(left+right)/2.,right-left))
                    },
                    Err(e) => {
                        self2.1.messages.send(e);
                        None
                    }
                };
                promise.satisfy(result);
                Ok(())
            }),
            stats: false
        });
        async_complete_task(&self.1.commander,&self.1.messages,handle, |e| (e,false));
    }
}
