use std::collections::HashMap;
use crate::core::channel::Channel;
use crate::shapeload::programloader::ProgramLoader;
use crate::{PeregrineCoreBase};
use crate::core::{ StickId, Stick };
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::any::Any;
use crate::util::message::DataMessage;
use crate::shapeload::programname::ProgramName;
use crate::util::builder::Builder;

#[derive(Clone)]
pub struct Authority {
    startup_program_name: ProgramName,
    jump_program_name: ProgramName,
    lookup_program_name: ProgramName
}

impl Authority {
    pub fn new(channel: &Channel, startup_program_name: &str, lookup_program_name: &str, jump_program_name: &str) -> Authority {
        Authority {
            startup_program_name: ProgramName(channel.clone(),startup_program_name.to_string()),
            lookup_program_name: ProgramName(channel.clone(),lookup_program_name.to_string()),
            jump_program_name: ProgramName(channel.clone(),jump_program_name.to_string())
        }
    }

    pub fn startup_program(&self) -> &ProgramName { &self.startup_program_name }
    pub fn lookup_program(&self) -> &ProgramName { &self.lookup_program_name }

    async fn run_startup_program(&self, base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> Result<(),DataMessage> {
        base.dauphin.run_program(&program_loader.clone(),PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.startup_program_name.clone(),
            payloads: None
        }).await?;
        base.queue.regenerate_track_config();
        Ok(())
    }

    fn preload_lookup_program(&self, base: &PeregrineCoreBase, program_loader: &ProgramLoader) {
        if !base.dauphin.is_present(&self.lookup_program_name) {
            program_loader.load_background(base,&self.lookup_program_name);
        }
    }

    pub async fn try_lookup(&self, dauphin: PgDauphin, program_loader: &ProgramLoader, id: StickId) -> Result<Vec<Stick>,DataMessage> {
        let sticks = Builder::new(vec![] as Vec<Stick>);
        let mut payloads = HashMap::new();
        payloads.insert("stick_id".to_string(),Box::new(id) as Box<dyn Any>);
        payloads.insert("sticks".to_string(),Box::new(sticks.clone()) as Box<dyn Any>);
        dauphin.run_program(program_loader,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.lookup_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(sticks.build())
    }

    pub async fn try_jump(&self, dauphin: PgDauphin, program_loader: &ProgramLoader, location: &str) -> Result<Vec<(String,(String,u64,u64))>,DataMessage> {
        let jumps = Builder::new(vec![] as Vec<(String,(String,u64,u64))>);
        let mut payloads = HashMap::new();
        payloads.insert("location".to_string(),Box::new(location.to_string()) as Box<dyn Any>);
        payloads.insert("jumps".to_string(),Box::new(jumps.clone()) as Box<dyn Any>);
        dauphin.run_program(program_loader,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.jump_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(jumps.build())
    }
}

pub(super) async fn load_stick_authority(base: &PeregrineCoreBase, program_loader: &ProgramLoader, channel: Channel) -> Result<Authority,DataMessage> {
    let backend = base.all_backends.backend(&channel);
    let stick_authority = backend.authority().await?;
    stick_authority.preload_lookup_program(base,program_loader);
    stick_authority.run_startup_program(base,program_loader).await?;
    Ok(stick_authority)
}
