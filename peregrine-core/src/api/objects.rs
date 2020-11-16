use crate::core::{ Viewport };
use crate::request::bootstrap::bootstrap;
use crate::train::{ TrainSet };
use crate::api::PeregrineIntegration;
use peregrine_dauphin_queue::{ PgDauphinQueue };
use crate::request::channel::Channel;
use std::sync::{ Arc, Mutex };
use crate::{ 
    PgCommander, PgDauphin, ProgramLoader, RequestManager, StickStore, StickAuthorityStore, Commander,
    CountingPromise, PanelProgramStore, PanelRunStore, PanelStore, DataStore
};

#[derive(Clone)]
pub struct PeregrineObjects {
    pub booted: CountingPromise,
    pub commander: PgCommander,
    pub data_store: DataStore,
    pub program_loader: ProgramLoader,
    pub manager: RequestManager,
    pub panel_program_store: PanelProgramStore,
    pub panel_run_store: PanelRunStore,
    pub panel_store: PanelStore,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub train_set: TrainSet,
    pub stick_store: StickStore,
    pub stick_authority_store: StickAuthorityStore,
    pub viewport: Viewport
}

impl PeregrineObjects {
    pub fn new<M>(integration: Box<dyn PeregrineIntegration>, commander: M, dauphin_queue: PgDauphinQueue) -> anyhow::Result<PeregrineObjects> 
                where M: Commander + 'static {
        let commander = PgCommander::new(Box::new(commander));
        let manager = RequestManager::new(integration.channel(),&commander);
        let data_store = DataStore::new(32,&commander,&manager);
        let booted = CountingPromise::new();
        let dauphin = PgDauphin::new(&dauphin_queue)?;
        let program_loader = ProgramLoader::new(&commander,&manager,&dauphin);
        let stick_authority_store = StickAuthorityStore::new(&commander,&manager,&program_loader,&dauphin);
        let stick_store = StickStore::new(&commander,&stick_authority_store,&booted)?;
        let panel_program_store = PanelProgramStore::new();
        let panel_run_store = PanelRunStore::new(32,&commander,&dauphin,&program_loader,&stick_store,&panel_program_store,&booted);
        let panel_store = PanelStore::new(128,&commander,&panel_run_store);
        Ok(PeregrineObjects {
            booted,
            commander,
            data_store,
            manager,
            panel_store,
            panel_program_store,
            panel_run_store,
            integration: Arc::new(Mutex::new(integration)),
            train_set: TrainSet::new(),
            stick_store,
            stick_authority_store,
            program_loader,
            viewport: Viewport::empty()
        })
    }

    // XXX move to API
    pub fn bootstrap(&self, dauphin: &PgDauphin, channel: Channel) -> anyhow::Result<()> {
        bootstrap(&self.manager,&self.program_loader,&self.commander,dauphin,channel,&self.booted)
    }
}
