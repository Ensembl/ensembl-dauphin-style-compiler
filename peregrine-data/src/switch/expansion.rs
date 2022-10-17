use std::sync::{Arc, Mutex};

use hashbrown::HashSet;
use peregrine_toolkit::{error::Error, lock};

use crate::{BackendNamespace, AllBackends};

#[derive(Clone)]
pub struct Expansion {
    name: String,
    fused: Arc<Mutex<HashSet<String>>>,
    backend_namespace: BackendNamespace
}

impl Expansion {
    pub(crate) fn new(name: &str, backend_namespace: &BackendNamespace) -> Expansion {
        Expansion {
            name: name.to_string(),
            fused: Arc::new(Mutex::new(HashSet::new())),
            backend_namespace: backend_namespace.clone()
        }
    }

    pub(crate) async fn run(&self, all_backends: &AllBackends, step: &str) -> Result<(),Error> {
        let mut fused = lock!(self.fused);
        if fused.contains(step) { return Ok(()); }
        fused.insert(step.to_string());
        drop(fused);
        let backend = all_backends.backend(&self.backend_namespace)?;
        backend.expand(&self.name,step).await?;
        Ok(())
    }
}
