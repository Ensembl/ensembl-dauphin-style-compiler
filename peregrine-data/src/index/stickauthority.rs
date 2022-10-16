use peregrine_toolkit::error::Error;
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::{PeregrineCoreBase, BackendNamespace, AllBackends};
use crate::core::{ StickId, Stick };

#[derive(Clone)]
pub struct Authority {
}

impl Authority {
    pub fn new() -> Authority {
        Authority {
        }
    }

    pub async fn try_lookup(&self, all_backends: AllBackends, channel_registry: &ChannelRegistry, stick_id: StickId) -> Result<Vec<Stick>,Error> {
        for backend_namespace in channel_registry.all() {
            let backend = all_backends.backend(&backend_namespace)?;
            if let Some(stick) = backend.stick(&stick_id).await? {
                return Ok(vec![stick]);
            }
        }
        Err(Error::operr(&format!("no such stick: {}",stick_id.get_id())))
    }

    pub async fn try_jump(&self, all_backends: &AllBackends, channel_registry: &ChannelRegistry, location: &str) -> Result<Vec<(String,(String,u64,u64))>,Error> {
        for backend_namespace in channel_registry.all().iter() {
            let backend = all_backends.backend(backend_namespace)?;
            if let Some(jump_location) = backend.jump(location).await? {
                return Ok(vec![(location.to_string(),(jump_location.stick,jump_location.left,jump_location.right))]);
            }
        }
        Ok(vec![])
    }
}

pub(super) async fn load_stick_authority(base: &PeregrineCoreBase, channel: BackendNamespace) -> Result<Authority,Error> {
    let backend = base.all_backends.backend(&channel)?;
    let stick_authority = backend.authority().await?;
    Ok(stick_authority)
}
