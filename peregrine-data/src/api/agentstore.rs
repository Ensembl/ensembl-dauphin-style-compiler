use std::sync::{ Arc, Mutex };
use commander::PromiseFuture;
use commander::FusePromise;
use crate::{ ProgramLoader, StickAuthorityStore, StickStore, LaneProgramLookup, LaneStore, DataStore };

#[derive(Clone)]
struct DelayedLoader<T> where T: Clone {
    item: Arc<Mutex<Option<T>>>,
    fuse: FusePromise<()>
}

impl<T> DelayedLoader<T> where T: Clone {
    fn new() -> DelayedLoader<T> {
        DelayedLoader {
            item: Arc::new(Mutex::new(None)),
            fuse: FusePromise::new()
        }
    }

    async fn get(&self) -> T {
        let promise = PromiseFuture::new();
        self.fuse.add(promise.clone());                   
        promise.await;
        self.item.lock().unwrap().as_ref().unwrap().clone()
    }

    fn set(&mut self, value: T) {
        self.item.lock().unwrap().replace(value);
        self.fuse.fuse(());
    }

    fn ready(&self) -> bool { self.item.lock().unwrap().is_some() }
}

#[derive(Clone)]
pub struct AgentStore {
    program_loader: DelayedLoader<ProgramLoader>,
    stick_authority_store: DelayedLoader<StickAuthorityStore>,
    stick_store: DelayedLoader<StickStore>,
    lane_store: DelayedLoader<LaneStore>,
    lane_program_lookup: DelayedLoader<LaneProgramLookup>,
    data_store: DelayedLoader<DataStore>
}

impl AgentStore {
    pub fn new() -> AgentStore {
        AgentStore {
            program_loader: DelayedLoader::new(),
            stick_authority_store: DelayedLoader::new(),
            lane_store: DelayedLoader::new(),
            lane_program_lookup: DelayedLoader::new(),
            stick_store: DelayedLoader::new(),
            data_store: DelayedLoader::new()
        }
    }

    pub fn set_program_loader(&mut self, agent: ProgramLoader) { self.program_loader.set(agent); }
    pub async fn program_loader(&self) -> ProgramLoader { self.program_loader.get().await }

    pub fn set_stick_authority_store(&mut self, agent: StickAuthorityStore) { self.stick_authority_store.set(agent); }
    pub async fn stick_authority_store(&self) -> StickAuthorityStore { self.stick_authority_store.get().await }

    pub fn set_lane_store(&mut self, agent: LaneStore) { self.lane_store.set(agent); }
    pub async fn lane_store(&self) -> LaneStore { self.lane_store.get().await }

    pub fn set_lane_program_lookup(&mut self, agent: LaneProgramLookup) { self.lane_program_lookup.set(agent); }
    pub async fn lane_program_lookup(&self) -> LaneProgramLookup { self.lane_program_lookup.get().await }

    pub fn set_stick_store(&mut self, agent: StickStore) { self.stick_store.set(agent); }
    pub async fn stick_store(&self) -> StickStore { self.stick_store.get().await }

    pub fn set_data_store(&mut self, agent: DataStore) { self.data_store.set(agent); }
    pub async fn data_store(&self) -> DataStore { self.data_store.get().await }

    pub fn ready(&self) -> bool {
        self.program_loader.ready() && self.stick_authority_store.ready() &&
        self.lane_store.ready() && self.stick_store.ready() && self.lane_program_lookup.ready() &&
        self.data_store.ready()
    }
}
