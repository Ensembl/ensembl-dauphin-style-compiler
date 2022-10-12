use std::collections::HashMap;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::log;

use crate::core::channel::channelregistry::ChannelRegistry;
use crate::shapeload::programloader::ProgramLoader;
use crate::{PeregrineCoreBase, BackendNamespace, AllBackends};
use crate::core::{ StickId, Stick };
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::any::Any;
use crate::util::message::DataMessage;
use crate::shapeload::programname::ProgramName;
use crate::util::builder::Builder;

#[derive(Clone)]
pub struct Authority {
    startup_program_name: ProgramName,
    lookup_program_name: ProgramName
}

impl Authority {
    pub fn new(channel: &BackendNamespace, startup_program_name: &str, lookup_program_name: &str) -> Authority {
        Authority {
            startup_program_name: ProgramName(channel.clone(),startup_program_name.to_string()),
            lookup_program_name: ProgramName(channel.clone(),lookup_program_name.to_string()),
        }
    }

    pub fn startup_program(&self) -> &ProgramName { &self.startup_program_name }
    pub fn lookup_program(&self) -> &ProgramName { &self.lookup_program_name }

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

    fn preload_lookup_program(&self, base: &PeregrineCoreBase, program_loader: &ProgramLoader) {
        if !base.dauphin.is_present(&self.lookup_program_name) {
            program_loader.load_background(base,&self.lookup_program_name);
        }
    }

    pub async fn try_lookup(&self, dauphin: PgDauphin, program_loader: &ProgramLoader, channel_registry: &ChannelRegistry, id: StickId) -> Result<Vec<Stick>,DataMessage> {
        let sticks = Builder::new(vec![] as Vec<Stick>);
        let mut payloads = HashMap::new();
        payloads.insert("stick_id".to_string(),Box::new(id) as Box<dyn Any>);
        payloads.insert("sticks".to_string(),Box::new(sticks.clone()) as Box<dyn Any>);
        dauphin.run_program(program_loader,channel_registry,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.lookup_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(sticks.build())
    }

    pub async fn try_jump(&self, all_backends: &AllBackends, channel_registry: &ChannelRegistry, location: &str) -> Result<Vec<(String,(String,u64,u64))>,DataMessage> {
        for backend_namespace in channel_registry.all().iter() {
            let backend = all_backends.backend(backend_namespace).map_err(|x| DataMessage::XXXTransitional(x))?;
            log!("trying {}",backend_namespace);
            if let Some(jump_location) = backend.jump(location).await.map_err(|e| DataMessage::XXXTransitional(e))? {
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
    stick_authority.preload_lookup_program(base,program_loader);
    stick_authority.run_startup_program(base,program_loader).await?;
    Ok(stick_authority)
}
