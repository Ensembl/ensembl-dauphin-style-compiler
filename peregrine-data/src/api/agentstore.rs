use std::sync::{ Arc, Mutex };
use commander::PromiseFuture;
use commander::FusePromise;
use crate::{ ProgramLoader, StickAuthorityStore, PanelRunStore, StickStore, PanelProgramStore, PanelStore, DataStore };

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
        use web_sys::console;
        console::log_1(&format!("inject").into());
        self.item.lock().unwrap().replace(value);
        self.fuse.fuse(());
    }

    fn ready(&self) -> bool { self.item.lock().unwrap().is_some() }
}

#[derive(Clone)]
pub struct AgentStore {
    program_loader: DelayedLoader<ProgramLoader>,
    stick_authority_store: DelayedLoader<StickAuthorityStore>,
    panel_run_store: DelayedLoader<PanelRunStore>,
    stick_store: DelayedLoader<StickStore>,
    panel_store: DelayedLoader<PanelStore>,
    panel_program_store: DelayedLoader<PanelProgramStore>,
    data_store: DelayedLoader<DataStore>
}

impl AgentStore {
    pub fn new() -> AgentStore {
        AgentStore {
            program_loader: DelayedLoader::new(),
            stick_authority_store: DelayedLoader::new(),
            panel_run_store: DelayedLoader::new(),
            panel_store: DelayedLoader::new(),
            panel_program_store: DelayedLoader::new(),
            stick_store: DelayedLoader::new(),
            data_store: DelayedLoader::new(),
        }
    }

    pub fn set_program_loader(&mut self, agent: ProgramLoader) { self.program_loader.set(agent); }
    pub async fn program_loader(&self) -> ProgramLoader { self.program_loader.get().await }

    pub fn set_stick_authority_store(&mut self, agent: StickAuthorityStore) { self.stick_authority_store.set(agent); }
    pub async fn stick_authority_store(&self) -> StickAuthorityStore { self.stick_authority_store.get().await }

    pub fn set_panel_run_store(&mut self, agent: PanelRunStore) { self.panel_run_store.set(agent); }
    pub async fn panel_run_store(&self) -> PanelRunStore { self.panel_run_store.get().await }

    pub fn set_panel_store(&mut self, agent: PanelStore) { self.panel_store.set(agent); }
    pub async fn panel_store(&self) -> PanelStore { self.panel_store.get().await }

    pub fn set_panel_program_store(&mut self, agent: PanelProgramStore) { self.panel_program_store.set(agent); }
    pub async fn panel_program_store(&self) -> PanelProgramStore { self.panel_program_store.get().await }

    pub fn set_stick_store(&mut self, agent: StickStore) { self.stick_store.set(agent); }
    pub async fn stick_store(&self) -> StickStore { self.stick_store.get().await }

    pub fn set_data_store(&mut self, agent: DataStore) { self.data_store.set(agent); }
    pub async fn data_store(&self) -> DataStore { self.data_store.get().await }

    pub fn ready(&self) -> bool {
        self.program_loader.ready() && self.stick_authority_store.ready() && self.panel_run_store.ready() &&
        self.panel_store.ready() && self.stick_store.ready() && self.panel_program_store.ready() &&
        self.data_store.ready()
    }
}
