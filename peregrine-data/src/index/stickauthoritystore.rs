use blackbox::blackbox_log;
use crate::{AgentStore, PeregrineCoreBase, lock};
use crate::request::{ Channel };
use super::stickauthority::{ StickAuthority, load_stick_authority };
use crate::core::{ StickId };
use std::sync::{ Arc, Mutex };
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

    fn add(&mut self, stick_authority: StickAuthority) {
        blackbox_log!("stickauthority","added stick authoritystartup={} lookup={}",
                        stick_authority.startup_program(),stick_authority.lookup_program());
        self.authorities.push(stick_authority);
    }

    fn each(&self) -> impl Iterator<Item=&StickAuthority> {
        self.authorities.iter()
    }
}

#[derive(Clone)]
pub struct StickAuthorityStore {
    base: PeregrineCoreBase,
    agent_store: AgentStore,
    data: Arc<Mutex<StickAuthorityStoreData>>
}

// TODO API-supplied stick authorities

impl StickAuthorityStore {
    pub fn new(base: &PeregrineCoreBase, agent_store: &AgentStore) -> StickAuthorityStore {
        StickAuthorityStore {
            base: base.clone(),
            agent_store: agent_store.clone(),
            data: Arc::new(Mutex::new(StickAuthorityStoreData::new()))
        }
    }

    pub async fn add(&self, channel: Channel) -> Result<(),DataMessage> {
        let stick_authority = load_stick_authority(&self.base,&self.agent_store,channel).await?;
        lock!(self.data).add(stick_authority);
        Ok(())

    }

    pub async fn try_lookup(&self, stick_id: StickId) -> Result<(),DataMessage> {
        let authorities : Vec<_> = lock!(self.data).each().cloned().collect(); // as we will be waiting and don't want the lock
        for a in &authorities {
            a.try_lookup(self.base.dauphin.clone(),&self.agent_store,stick_id.clone()).await.unwrap_or(());
        }
        Ok(())
    }
}
