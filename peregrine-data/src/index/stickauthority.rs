use std::collections::HashMap;
use crate::{PeregrineCoreBase };
use crate::core::{ StickId, Stick };
use crate::request::stickauthority::get_stick_authority;
use crate::request::{ Channel };
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::any::Any;
use crate::util::message::DataMessage;
use crate::api::{ AgentStore, ApiMessage };
use crate::lane::programname::ProgramName;
use crate::util::builder::Builder;

#[derive(Clone)]
pub struct StickAuthority {
    startup_program_name: ProgramName,
    lookup_program_name: ProgramName
}

impl StickAuthority {
    pub fn new(channel: &Channel, startup_program_name: &str, lookup_program_name: &str) -> StickAuthority {
        StickAuthority {
            startup_program_name: ProgramName(channel.clone(),startup_program_name.to_string()),
            lookup_program_name: ProgramName(channel.clone(),lookup_program_name.to_string())
        }
    }

    pub fn startup_program(&self) -> &ProgramName { &self.startup_program_name }
    pub fn lookup_program(&self) -> &ProgramName { &self.lookup_program_name }

    async fn run_startup_program(&self, base: &PeregrineCoreBase, agent_store: &AgentStore) -> Result<(),DataMessage> {
        base.dauphin.run_program(&agent_store.program_loader().await.clone(),PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.startup_program_name.clone(),
            payloads: None
        }).await?;
        base.queue.push(ApiMessage::RegeneraateTrackConfig);
        Ok(())
    }

    async fn preload_lookup_program(&self, base: &PeregrineCoreBase, agent_store: &AgentStore) {
        if !base.dauphin.is_present(&self.lookup_program_name) {
            agent_store.program_loader().await.load_background(base,&self.lookup_program_name);
        }
    }

    pub async fn try_lookup(&self, dauphin: PgDauphin, agent_store: &AgentStore, id: StickId) -> Result<Vec<Stick>,DataMessage> {
        let sticks = Builder::new(vec![] as Vec<Stick>);
        let mut payloads = HashMap::new();
        payloads.insert("stick_id".to_string(),Box::new(id) as Box<dyn Any>);
        payloads.insert("sticks".to_string(),Box::new(sticks.clone()) as Box<dyn Any>);
        dauphin.run_program(&agent_store.program_loader().await,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            program_name: self.lookup_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(sticks.build())
    }
}

pub(super) async fn load_stick_authority(base: &PeregrineCoreBase, agent_store: &AgentStore, channel: Channel) -> Result<StickAuthority,DataMessage> {
    let stick_authority = get_stick_authority(base.manager.clone(),channel.clone()).await?;
    stick_authority.preload_lookup_program(base,agent_store).await;
    stick_authority.run_startup_program(base,agent_store).await?;
    Ok(stick_authority)
}
