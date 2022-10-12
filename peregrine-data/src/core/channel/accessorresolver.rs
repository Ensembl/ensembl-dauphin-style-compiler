use peregrine_toolkit::error::Error;
use super::{channelregistry::ChannelRegistry, backendnamespace::BackendNamespace};

fn matches_self(accessor: &str) -> bool {
    accessor == "self()" || accessor.starts_with("self://")
}

#[derive(Clone)]
pub struct AccessorResolver {
    registry: ChannelRegistry,
    base: BackendNamespace
}

impl AccessorResolver {
    pub fn new(registry: &ChannelRegistry, base: &BackendNamespace) -> AccessorResolver {
        AccessorResolver {
            registry: registry.clone(),
            base: base.clone()
        }
    }

    pub async fn resolve(&self, accessor: &str) -> Result<BackendNamespace,Error> {
        if matches_self(accessor) {
            Ok(self.base.clone())
        } else {
            self.registry.spec_to_name(accessor).await
        }
    }
}
