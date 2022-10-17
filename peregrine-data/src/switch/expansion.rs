use peregrine_toolkit::error::Error;

use crate::{BackendNamespace, AllBackends};

#[derive(Clone)]
pub struct Expansion {
    name: String,
    backend_namespace: BackendNamespace
}

impl Expansion {
    pub(crate) fn new(name: &str, backend_namespace: &BackendNamespace) -> Expansion {
        Expansion { name: name.to_string(), backend_namespace: backend_namespace.clone() }
    }

    pub(crate) async fn run(&self, all_backends: &AllBackends, step: &str) -> Result<(),Error> {
        let backend = all_backends.backend(&self.backend_namespace)?;
        backend.expand(&self.name,step).await?;
        Ok(())
    }
}
