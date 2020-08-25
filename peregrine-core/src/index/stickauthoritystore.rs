use blackbox::blackbox_log;
use crate::lock;
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::request::stickauthority::get_stick_authority_program;
use super::stickauthority::StickAuthority;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::sync::{ Arc, Mutex };

struct StickAuthorityStoreData {
    authorities: Vec<StickAuthority>
}

impl StickAuthorityStoreData {
    fn new() -> StickAuthorityStoreData {
        StickAuthorityStoreData {
            authorities: vec![]
        }
    }

    fn add(&mut self, channel: &Channel) {
        blackbox_log!("stickauthority","added stick authority at {}",channel.to_string());
        //self.authorities.push(StickAuthority::new(channel,name))
    }
}

#[derive(Clone)]
pub struct StickAuthorityStore(Arc<Mutex<StickAuthorityStoreData>>);

impl StickAuthorityStore {
    pub fn new() -> StickAuthorityStore {
        StickAuthorityStore(Arc::new(Mutex::new(StickAuthorityStoreData::new())))
    }

    pub fn add(&self, channel: &Channel) {
        lock!(self.0).add(channel)
    }
}

pub async fn add_stick_authority(manager: &RequestManager, loader: &ProgramLoader, dauphin: &PgDauphin, channel: &Channel) -> anyhow::Result<()> {
    let (prog_channel,prog_name) = get_stick_authority_program(manager.clone(),channel.clone()).await?;
    dauphin.run_program(loader,PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        channel: prog_channel,
        program_name: prog_name
    }).await?;
    Ok(())
}
