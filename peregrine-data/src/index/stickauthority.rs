use std::collections::HashMap;
use crate::{PeregrineCoreBase };
use crate::core::{ StickId };
use crate::request::stickauthority::get_stick_authority;
use crate::request::{ Channel };
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::any::Any;
use crate::util::message::DataMessage;
use crate::api::AgentStore;

#[derive(Clone)]
pub struct StickAuthority {
    channel: Channel,
    startup_program_name: String,
    lookup_program_name: String
}

impl StickAuthority {
    pub fn new(channel: &Channel, startup_program_name: &str, lookup_program_name: &str) -> StickAuthority {
        StickAuthority {
            channel: channel.clone(),
            startup_program_name: startup_program_name.to_string(),
            lookup_program_name: lookup_program_name.to_string(),
        }
    }

    pub fn channel(&self) -> &Channel { &self.channel }
    pub fn startup_program(&self) -> &str { &self.startup_program_name }
    pub fn lookup_program(&self) -> &str { &self.lookup_program_name }

    async fn run_startup_program(&self, base: &PeregrineCoreBase, agent_store: &AgentStore) -> Result<(),DataMessage> {
        base.dauphin.run_program(&agent_store.program_loader().await.clone(),PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            channel: self.channel.clone(),
            program_name: self.startup_program_name.clone(),
            payloads: None
        }).await   
    }

    async fn preload_lookup_program(&self, base: &PeregrineCoreBase, agent_store: &AgentStore) {
        if !base.dauphin.is_present(&self.channel,&self.lookup_program_name) {
            agent_store.program_loader().await.load_background(&self.channel,&self.lookup_program_name);
        }
    }

    pub async fn try_lookup(&self, dauphin: PgDauphin, agent_store: &AgentStore, id: StickId) -> Result<(),DataMessage> {
        let mut payloads = HashMap::new();
        payloads.insert("stick_id".to_string(),Box::new(id) as Box<dyn Any>);
        dauphin.run_program(&agent_store.program_loader().await,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            channel: self.channel.clone(),
            program_name: self.lookup_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(())
    }
}

pub(super) async fn load_stick_authority(base: &PeregrineCoreBase, agent_store: &AgentStore, channel: Channel) -> Result<StickAuthority,DataMessage> {
    let stick_authority = get_stick_authority(base.manager.clone(),channel.clone()).await?;
    stick_authority.preload_lookup_program(base,agent_store).await;
    stick_authority.run_startup_program(base,agent_store).await?;
    Ok(stick_authority)
}
