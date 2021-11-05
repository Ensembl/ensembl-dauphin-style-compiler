use std::sync::{ Arc, Mutex };
use commander::PromiseFuture;
use commander::FusePromise;
use crate::PeregrineCoreBase;
use crate::index::jumpstore::JumpStore;
use crate::lane::programloader::ProgramLoader;
use crate::{ AuthorityStore, StickStore, LaneStore, DataStore };

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
    pub program_loader: ProgramLoader,
    pub stick_authority_store: AuthorityStore,
    pub stick_store: StickStore,
    pub jump_store: JumpStore,
    pub lane_store: LaneStore,
    pub data_store: DataStore
}

impl AgentStore {
    pub fn new(base: &PeregrineCoreBase) -> AgentStore {
        /* Payloads are about 4k on the wire, maybe 4x that unpacked. 1000 => ~64Mb. */
        let data_store = DataStore::new(10240,&base);
        let program_loader = ProgramLoader::new(&base);
        let stick_authority_store = AuthorityStore::new(&base,&program_loader);
        let stick_store = StickStore::new(&base,&stick_authority_store);
        let lane_store = LaneStore::new(1024,&base,&program_loader);
        let jump_store = JumpStore::new(&base,&stick_authority_store);
        AgentStore {
            program_loader, stick_authority_store, stick_store, jump_store, lane_store, data_store
        }
    }
}
