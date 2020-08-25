use blackbox::blackbox_log;
use crate::lock;
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::request::stickauthority::get_stick_authority;
use super::stickauthority::StickAuthority;
use crate::run::{ PgDauphin, PgDauphinTaskSpec };
use std::sync::{ Arc, Mutex };
use crate::{ PgCommander, PgCommanderTaskSpec };

struct StickAuthorityStoreData {
    authorities: Vec<StickAuthority>
}

impl StickAuthorityStoreData {
    fn new() -> StickAuthorityStoreData {
        StickAuthorityStoreData {
            authorities: vec![]
        }
    }

    fn add(&mut self, stickauthority: StickAuthority) {
        blackbox_log!("stickauthority","added stick authority channel={} startup={} lookup={}",
                        stickauthority.channel(),stickauthority.startup_program(),stickauthority.lookup_program());
        self.authorities.push(stickauthority);
    }
}

#[derive(Clone)]
pub struct StickAuthorityStore {
    commander: PgCommander,
    manager: RequestManager,
    loader: ProgramLoader,
    dauphin: PgDauphin,
    data: Arc<Mutex<StickAuthorityStoreData>>
}

impl StickAuthorityStore {
    pub fn new(commander: &PgCommander, manager: &RequestManager, loader: &ProgramLoader, dauphin: &PgDauphin) -> StickAuthorityStore {
        StickAuthorityStore {
            commander: commander.clone(),
            manager: manager.clone(),
            loader: loader.clone(),
            dauphin: dauphin.clone(),
            data: Arc::new(Mutex::new(StickAuthorityStoreData::new()))
        }
    }

    pub fn add(&self, channel: &Channel) -> anyhow::Result<()> {
        let channel = channel.clone();
        let manager = self.manager.clone();
        let loader = self.loader.clone();
        let dauphin = self.dauphin.clone();
        let data = self.data.clone();
        self.commander.add_task(PgCommanderTaskSpec {
            name: format!("stick authority loader: {}",channel.to_string()),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(add_stick_authority(manager,loader,dauphin,data,channel))
        });
        Ok(())
    }
}

async fn add_stick_authority(manager: RequestManager, loader: ProgramLoader, dauphin: PgDauphin, data: Arc<Mutex<StickAuthorityStoreData>>, channel: Channel) -> anyhow::Result<()> {
    let stick_authority = get_stick_authority(manager.clone(),channel.clone()).await?;
    let channel = stick_authority.channel().clone();
    let program_name = stick_authority.startup_program().to_string();
    let lookup_name = stick_authority.lookup_program().to_string();
    lock!(data).add(stick_authority);
    dauphin.run_program(&loader,PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        channel: channel.clone(),
        program_name
    }).await?;
    loader.load_background(&channel,&lookup_name)?;
    Ok(())
}
