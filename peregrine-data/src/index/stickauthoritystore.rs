use anyhow::Context;
use blackbox::blackbox_log;
use crate::lock;
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::request::stickauthority::get_stick_authority;
use super::stickauthority::StickAuthority;
use crate::run::{ PgDauphin, PgDauphinTaskSpec, add_task };
use crate::core::{ StickId };
use std::sync::{ Arc, Mutex };
use crate::{ PgCommander, PgCommanderTaskSpec, CountingPromise };
use crate::util::message::DataMessage;

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

    fn each(&self) -> impl Iterator<Item=&StickAuthority> {
        self.authorities.iter()
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

    pub fn add(&self, channel: &Channel, booted: &CountingPromise) -> Result<(),DataMessage> {
        let channel = channel.clone();
        let manager = self.manager.clone();
        let loader = self.loader.clone();
        let dauphin = self.dauphin.clone();
        let data = self.data.clone();
        booted.lock();
        let booted = booted.clone();
        add_task(&self.commander,PgCommanderTaskSpec {
            name: format!("stick authority loader: {}",channel.to_string()),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                add_stick_authority(manager,loader,dauphin,data,channel).await?;
                booted.unlock();
                Ok(())
            })
        });
        Ok(())
    }

    pub async fn try_lookup(&self, stick_id: StickId) -> anyhow::Result<()> {
        let authorities : Vec<_> = lock!(self.data).each().cloned().collect(); // as we will be waiting and don't want the lock
        for a in &authorities {
            a.try_lookup(self.dauphin.clone(),self.loader.clone(),stick_id.clone()).await.unwrap_or(());
        }
        Ok(())
    }
}

async fn add_stick_authority(manager: RequestManager, loader: ProgramLoader, dauphin: PgDauphin, data: Arc<Mutex<StickAuthorityStoreData>>, channel: Channel) -> Result<(),DataMessage> {
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
        program_name,
        payloads: None
    }).await.map_err(|e| e.to_string()).map_err(|e| DataMessage::XXXTmp(e.to_string()))?;
    if !dauphin.is_present(&channel,&lookup_name) {
        loader.load_background(&channel,&lookup_name).map_err(|e| DataMessage::XXXTmp(e.to_string()))?;
    }
    Ok(())
}
