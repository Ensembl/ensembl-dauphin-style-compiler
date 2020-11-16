use crate::core::{ Viewport };
use crate::train::{ TrainSet };
use crate::api::PeregrineIntegration;
use peregrine_dauphin_queue::{ PgDauphinQueue };
use std::sync::{ Arc, Mutex };
use crate::{ 
    PgCommander, PgDauphin, ProgramLoader, RequestManager, StickStore, StickAuthorityStore, Commander,
    CountingPromise, PanelProgramStore, PanelRunStore, PanelStore,ChannelIntegration
};

#[derive(Clone)]
pub struct PeregrineObjects {
    pub(crate) commander: PgCommander,
    pub(crate) panel_store: PanelStore,
    pub(crate) integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub(crate) train_set: TrainSet,
    pub(crate) stick_store: StickStore,
    pub(crate) viewport: Viewport
}

impl PeregrineObjects {
    pub fn new<M>(integration: Box<dyn PeregrineIntegration>, commander: M, dauphin_queue: PgDauphinQueue) -> anyhow::Result<PeregrineObjects> 
                where M: Commander + 'static {
        let commander = PgCommander::new(Box::new(commander));
        let manager = RequestManager::new(integration.channel(),&commander);
        let booted = CountingPromise::new();
        let dauphin = PgDauphin::new(&dauphin_queue)?;
        let loader = ProgramLoader::new(&commander,&manager,&dauphin);
        let stick_authority_store = StickAuthorityStore::new(&commander,&manager,&loader,&dauphin);
        let stick_store = StickStore::new(&commander,&stick_authority_store,&booted)?;
        let panel_program_store = PanelProgramStore::new();
        let panel_run_store = PanelRunStore::new(32,&commander,&dauphin,&loader,&stick_store,&panel_program_store,&booted);
        let panel_store = PanelStore::new(128,&commander,&panel_run_store);
        Ok(PeregrineObjects {
            commander,
            panel_store,
            integration: Arc::new(Mutex::new(integration)),
            train_set: TrainSet::new(),
            stick_store,
            viewport: Viewport::empty()
        })
    }
}