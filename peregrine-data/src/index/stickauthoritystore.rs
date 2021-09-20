use crate::{PeregrineCoreBase, ProgramLoader };
use crate::request::{ Channel };
use super::stickauthority::{ StickAuthority, load_stick_authority };
use crate::core::{ StickId, Stick };
use std::sync::{ Arc, Mutex };
use crate::util::message::DataMessage;
use peregrine_toolkit::lock;

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
        self.authorities.push(stick_authority);
    }

    fn each(&self) -> impl Iterator<Item=&StickAuthority> {
        self.authorities.iter()
    }
}

#[derive(Clone)]
pub struct StickAuthorityStore {
    base: PeregrineCoreBase,
    program_loader: ProgramLoader,
    data: Arc<Mutex<StickAuthorityStoreData>>
}

// TODO API-supplied stick authorities

impl StickAuthorityStore {
    pub fn new(base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> StickAuthorityStore {
        StickAuthorityStore {
            base: base.clone(),
            program_loader: program_loader.clone(),
            data: Arc::new(Mutex::new(StickAuthorityStoreData::new()))
        }
    }

    pub async fn add(&self, channel: Channel) -> Result<(),DataMessage> {
        let stick_authority = load_stick_authority(&self.base,&self.program_loader,channel).await?;
        lock!(self.data).add(stick_authority);
        Ok(())

    }

    pub async fn try_lookup(&self, stick_id: StickId) -> Result<Vec<Stick>,DataMessage> {
        let mut sticks = vec![];
        let authorities : Vec<_> = lock!(self.data).each().cloned().collect(); // as we will be waiting and don't want the lock
        for a in &authorities {
            let mut more = a.try_lookup(self.base.dauphin.clone(),&self.program_loader,stick_id.clone()).await?;
            sticks.append(&mut more);
        }
        Ok(sticks)
    }

    pub async fn try_location(&self, location: &str) -> Result<Option<(String,u64,u64)>,DataMessage> {
        let authorities : Vec<_> = lock!(self.data).each().cloned().collect(); // as we will be waiting and don't want the lock
        for a in &authorities {
            let more = a.try_jump(self.base.dauphin.clone(),&self.program_loader,location).await?;
            for (id,jump) in &more {
                if id == location {
                    return Ok(Some((jump.0.to_string(),jump.1,jump.2)));
                }
            }
        }
        Ok(None)
    }
}
