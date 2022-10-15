use peregrine_toolkit::error::Error;
use peregrine_toolkit::log;
use crate::core::channel::channelregistry::ChannelRegistry;
use crate::shapeload::programloader::ProgramLoader;
use crate::{PeregrineCoreBase, BackendNamespace, AllBackends};
use crate::core::{ StickId, Stick };
use crate::run::{ PgDauphinTaskSpec };
use crate::shapeload::programname::ProgramName;

#[derive(Clone)]
pub struct Authority {
    startup_program_name: ProgramName
}

impl Authority {
    pub fn new(channel: &BackendNamespace, startup_program_name: &str) -> Authority {
        Authority {
            startup_program_name: ProgramName(channel.clone(),startup_program_name.to_string()),
        }
    }

    pub fn startup_program(&self) -> &ProgramName { &self.startup_program_name }

    async fn run_startup_program(&self, base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> Result<(),Error> {
        base.dauphin.run_program(&program_loader.clone(),&base.channel_registry,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.startup_program_name.clone(),
            payloads: None
        }).await.map_err(|e| Error::operr(&format!("error running startup program: {}",e.to_string())))?;
        base.queue.regenerate_track_config();
        Ok(())
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
            log!("trying {}",backend_namespace);
            if let Some(jump_location) = backend.jump(location).await? {
                return Ok(vec![(location.to_string(),(jump_location.stick,jump_location.left,jump_location.right))]);
            }
        }
        log!("done trying");
        Ok(vec![])
    }
}

pub(super) async fn load_stick_authority(base: &PeregrineCoreBase, program_loader: &ProgramLoader, channel: BackendNamespace) -> Result<Authority,Error> {
    let backend = base.all_backends.backend(&channel)?;
    let stick_authority = backend.authority().await?;
    stick_authority.run_startup_program(base,program_loader).await?;
    Ok(stick_authority)
}
