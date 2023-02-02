use std::{sync::Arc, collections::HashMap};
use peregrine_toolkit::{error::Error, error, log};
use crate::{util::memoized::{Memoized, MemoizedType}, PeregrineCoreBase, AllBackends, core::channel::channelregistry::ChannelRegistry};

async fn get_small_values(all_backends: &AllBackends, channel_registry: &ChannelRegistry, namespace: &str, column: &str) -> Result<HashMap<String,String>,Error> {
    let mut out = HashMap::new();
    for backend_namespace in &channel_registry.all() {
        let backend = all_backends.backend(backend_namespace)?;
        let stick = backend.small_values(namespace,column).await?;
        out.extend(stick.iter().map(|(x,y)| (x.clone(),y.clone())));
    }
    Ok(out)
}

async fn query_small_values(all_backends: &AllBackends, channel_registry: &ChannelRegistry, namespace: &str, column: &str) -> Arc<HashMap<String,String>> {
    match get_small_values(all_backends,channel_registry,namespace,column).await {
        Ok(v) => Arc::new(v),
        Err(e) => {
            error!("{:?}",e);
            Arc::new(HashMap::new())
        }
    }
}

fn make_small_values_cache(all_backends: &AllBackends, channel_registry: &ChannelRegistry) -> Memoized<(String,String),Arc<HashMap<String,String>>> {
    let all_backends = all_backends.clone();
    let channel_registry = channel_registry.clone();
    Memoized::new(MemoizedType::Cache(100),move |_: &Memoized<(String,String),Arc<HashMap<String,String>>>,(namespace,column)| {
        let all_backends = all_backends.clone();
        let channel_registry = channel_registry.clone();    
        let namespace = namespace.to_string();
        let column = column.clone();
        Box::pin(async move { query_small_values(&all_backends,&channel_registry,&namespace,&column).await })
    })   
}


#[derive(Clone)]
pub struct SmallValuesStore {
    values: Memoized<(String,String),Arc<HashMap<String,String>>>,
    base: PeregrineCoreBase
}

impl SmallValuesStore {
    pub fn new(base: &PeregrineCoreBase) -> SmallValuesStore {
        SmallValuesStore {
            values: make_small_values_cache(&base.all_backends,&base.channel_registry),
            base: base.clone()
        }
    }

    pub async fn get(&self, namespace: &str, column: &str, row: &str) -> Result<Option<String>,Error> {
        self.base.booted.wait().await;
        let value_set = self.values.get(&(namespace.to_string(),column.to_string())).await.as_ref().clone();
        Ok(value_set.get(row).cloned())
    }
}
