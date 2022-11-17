use peregrine_toolkit::error::Error;
use super::{channelregistry::ChannelRegistry, backendnamespace::BackendNamespace};

fn matches_self(accessor: &str) -> bool {
    accessor == "self()" || accessor.starts_with("self://")
}

fn matches_source(accessor: &str) -> bool {
    accessor == "source()" || accessor.starts_with("source://")
}

#[derive(Clone)]
pub struct AccessorResolver {
    registry: ChannelRegistry,
    program_base: BackendNamespace,
    track_base: BackendNamespace
}

impl AccessorResolver {
    pub fn new(registry: &ChannelRegistry, program_base: &BackendNamespace, track_base: &BackendNamespace) -> AccessorResolver {
        AccessorResolver {
            registry: registry.clone(),
            program_base: program_base.clone(),
            track_base: track_base.clone()
        }
    }

    pub async fn resolve(&self, accessor: &str) -> Result<BackendNamespace,Error> {
        if matches_self(accessor) {
            Ok(self.track_base.clone())
        } else if matches_source(accessor) {
            Ok(self.program_base.clone())
        } else {
            self.registry.spec_to_name(accessor).await
        }
    }

    pub fn all(&self) -> Vec<BackendNamespace> {
        self.registry.all()
    }
}
