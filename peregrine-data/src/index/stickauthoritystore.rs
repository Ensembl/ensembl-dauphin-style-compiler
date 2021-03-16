use anyhow::Context;
use blackbox::blackbox_log;
use crate::{AgentStore, PeregrineCoreBase, api::MessageSender, lock};
use crate::request::{ Channel, RequestManager };
use crate::request::program::ProgramLoader;
use crate::request::stickauthority::get_stick_authority;
use super::stickauthority::StickAuthority;
use crate::run::{ PgDauphin, PgDauphinTaskSpec, add_task, async_complete_task };
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
    base: PeregrineCoreBase,
    data: Arc<Mutex<StickAuthorityStoreData>>
}

// TODO API-supplied stick authorities

impl StickAuthorityStore {
    pub fn new(base: &PeregrineCoreBase) -> StickAuthorityStore {
        StickAuthorityStore {
            base: base.clone(),
            data: Arc::new(Mutex::new(StickAuthorityStoreData::new()))
        }
    }

    pub fn add(&self, channel: &Channel, agent_store: &AgentStore, booted: &CountingPromise) -> Result<(),DataMessage> {
        let channel = channel.clone();
        let base = self.base.clone();
        let agent_store = agent_store.clone();
        let data = self.data.clone();
        booted.lock();
        let handle = add_task(&self.base.commander,PgCommanderTaskSpec {
            name: format!("stick authority loader: {}",channel.to_string()),
            prio: 2,
            slot: None,
            timeout: None,
            task: Box::pin(async move {
                add_stick_authority(base.manager,agent_store,base.dauphin,data,channel).await?;
                base.booted.unlock();
                Ok(())
            })
        });
        async_complete_task(&self.base.commander, &self.base.messages,handle, |e| {
            (DataMessage::StickAuthorityUnavailable(Box::new(e)),true)
        });
        Ok(())
    }

    pub async fn try_lookup(&self, agent_store: &AgentStore, stick_id: StickId) -> Result<(),DataMessage> {
        let authorities : Vec<_> = lock!(self.data).each().cloned().collect(); // as we will be waiting and don't want the lock
        for a in &authorities {
            a.try_lookup(self.base.dauphin.clone(),agent_store,stick_id.clone()).await.unwrap_or(());
        }
        Ok(())
    }
}

async fn add_stick_authority(manager: RequestManager, agent_store: AgentStore, dauphin: PgDauphin, data: Arc<Mutex<StickAuthorityStoreData>>, channel: Channel) -> Result<(),DataMessage> {
    let stick_authority = get_stick_authority(manager.clone(),channel.clone()).await?;
    let channel = stick_authority.channel().clone();
    let program_name = stick_authority.startup_program().to_string();
    let lookup_name = stick_authority.lookup_program().to_string();
    lock!(data).add(stick_authority);
    dauphin.run_program(&agent_store.program_loader().await.clone(),PgDauphinTaskSpec {
        prio: 2,
        slot: None,
        timeout: None,
        channel: channel.clone(),
        program_name,
        payloads: None
    }).await?;
    if !dauphin.is_present(&channel,&lookup_name) {
        agent_store.program_loader().await.load_background(&channel,&lookup_name).map_err(|e| DataMessage::XXXTmp(e.to_string()))?;
    }
    Ok(())
}
