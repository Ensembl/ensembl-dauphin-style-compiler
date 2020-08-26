use std::collections::HashMap;
use blackbox::blackbox_log;
use crate::lock;
use crate::core::{ Stick, StickId };
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::any::Any;
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct StickAuthority {
    channel: Channel,
    startup_program_name: String,
    resolution_program_name: String
}

impl StickAuthority {
    pub fn new(channel: &Channel, startup_program_name: &str, resolution_program_name: &str) -> StickAuthority {
        blackbox_log!("stickauthority","");
        StickAuthority {
            channel: channel.clone(),
            startup_program_name: startup_program_name.to_string(),
            resolution_program_name: resolution_program_name.to_string(),
        }
    }

    pub fn channel(&self) -> &Channel { &self.channel }
    pub fn startup_program(&self) -> &str { &self.startup_program_name }
    pub fn lookup_program(&self) -> &str { &self.resolution_program_name }

    pub async fn lookup(&self, dauphin: PgDauphin, loader: ProgramLoader, id: StickId) -> anyhow::Result<Option<Stick>> {
        let mut payloads = HashMap::new();
        payloads.insert("stick_id".to_string(),Box::new(id) as Box<dyn Any>);
        dauphin.run_program(&loader,PgDauphinTaskSpec {
            prio: 2,
            slot: None,
            timeout: None,
            channel: self.channel.clone(),
            program_name: self.resolution_program_name.clone(),
            payloads: Some(payloads)
        }).await?;
        Ok(None)
    }
}
