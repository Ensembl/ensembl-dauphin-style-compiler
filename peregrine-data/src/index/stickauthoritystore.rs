use crate::core::channel::Channel;
use crate::shapeload::programloader::ProgramLoader;
use crate::{PeregrineCoreBase };
use super::stickauthority::{ Authority, load_stick_authority };
use crate::core::{ StickId, Stick };
use std::sync::{ Arc, Mutex };
use crate::util::message::DataMessage;
use peregrine_toolkit::lock;

struct AuthorityStoreData {
    authorities: Vec<Authority>
}

impl AuthorityStoreData {
    fn new() -> AuthorityStoreData {
        AuthorityStoreData {
            authorities: vec![]
        }
    }

    fn add(&mut self, stick_authority: Authority) {
        self.authorities.push(stick_authority);
    }

    fn each(&self) -> impl Iterator<Item=&Authority> {
        self.authorities.iter()
    }
}

#[derive(Clone)]
pub struct AuthorityStore {
    base: PeregrineCoreBase,
    program_loader: ProgramLoader,
    data: Arc<Mutex<AuthorityStoreData>>
}

// TODO API-supplied stick authorities

impl AuthorityStore {
    pub fn new(base: &PeregrineCoreBase, program_loader: &ProgramLoader) -> AuthorityStore {
        AuthorityStore {
            base: base.clone(),
            program_loader: program_loader.clone(),
            data: Arc::new(Mutex::new(AuthorityStoreData::new()))
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
