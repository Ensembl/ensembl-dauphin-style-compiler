use crate::BackendNamespace;

#[derive(Clone)]
pub struct Expansion {
    name: String,
    backend_namespace: BackendNamespace
}

impl Expansion {
    pub(crate) fn new(name: &str, backend_namespace: &BackendNamespace) -> Expansion {
        Expansion { name: name.to_string(), backend_namespace: backend_namespace.clone() }
    }
}
