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
use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::core::{ Track, Focus, StickId };

#[derive(Clone)]
pub struct MessageSender(Arc<Mutex<Box<dyn FnMut(&str) + 'static + Send>>>);

impl MessageSender {
    pub(crate) fn new<F>(cb :F) -> MessageSender where F: FnMut(&str) + 'static + Send {
        MessageSender(Arc::new(Mutex::new(Box::new(cb))))
    }

    pub(crate) fn send(&self,message: &str) {
        (self.0.lock().unwrap())(message);
    }
}

#[derive(Clone)]
pub struct PeregrineCore {
    pub dauphin: PgDauphin,
    pub dauphin_queue: PgDauphinQueue,
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
    pub viewport: Viewport,
    pub queue: PeregrineApiQueue
}

impl PeregrineCore {
    pub fn new<M,F>(integration: Box<dyn PeregrineIntegration>, commander: M, messages: F) -> anyhow::Result<PeregrineCore> 
                where M: Commander + 'static, F: FnMut(&str) + 'static + Send {
        let messages = MessageSender(Arc::new(Mutex::new(Box::new(messages))));
        let dauphin_queue = PgDauphinQueue::new();
        let commander = PgCommander::new(Box::new(commander));
        let manager = RequestManager::new(integration.channel(),&commander,&messages);
        let data_store = DataStore::new(32,&commander,&manager);
        let booted = CountingPromise::new();
        let dauphin = PgDauphin::new(&dauphin_queue)?;
        let program_loader = ProgramLoader::new(&commander,&manager,&dauphin);
        let stick_authority_store = StickAuthorityStore::new(&commander,&manager,&program_loader,&dauphin);
        let stick_store = StickStore::new(&commander,&stick_authority_store,&booted)?;
        let panel_program_store = PanelProgramStore::new();
        let panel_run_store = PanelRunStore::new(32,&commander,&dauphin,&program_loader,&stick_store,&panel_program_store,&booted);
        let panel_store = PanelStore::new(128,&commander,&panel_run_store);
        Ok(PeregrineCore {
            booted,
            commander,
            data_store,
            dauphin,
            dauphin_queue,
            manager,
            panel_store,
            panel_program_store,
            panel_run_store,
            integration: Arc::new(Mutex::new(integration)),
            train_set: TrainSet::new(),
            stick_store,
            stick_authority_store,
            program_loader,
            viewport: Viewport::empty(),
            queue: PeregrineApiQueue::new()
        })
    }

    pub fn dauphin_ready(&mut self) {
        self.manager.add_receiver(Box::new(self.dauphin.clone()));
    }

    pub fn application_ready(&mut self) {
        self.queue.clone().run(self);
        self.queue.push(ApiMessage::Ready);
    }

    pub fn bootstrap(&self, channel: Channel) -> anyhow::Result<()> {
        bootstrap(&self.manager,&self.program_loader,&self.commander,&self.dauphin,channel,&self.booted)
    }

    /* from api */
    pub fn ready(&self, mut core: PeregrineCore) {
        self.queue.run(&mut core);
        self.queue.push(ApiMessage::Ready);
    }

    pub fn backend_bootstrap(&self, channel: Channel) {
        self.queue.push(ApiMessage::Bootstrap(channel));
    }

    pub fn transition_complete(&self) {
        self.queue.push(ApiMessage::TransitionComplete);
    }

    pub fn add_track(&self, track: Track) {
        self.queue.push(ApiMessage::AddTrack(track));
    }

    pub fn remove_track(&self, track: Track) {
        self.queue.push(ApiMessage::RemoveTrack(track));
    }

    pub fn set_position(&self, pos: f64) {
        self.queue.push(ApiMessage::SetPosition(pos));
    }

    pub fn set_scale(&self, scale: f64) {
        self.queue.push(ApiMessage::SetScale(scale));
    }

    pub fn set_focus(&self, focus: &Focus) {
        self.queue.push(ApiMessage::SetFocus(focus.clone()));
    }

    pub fn set_stick(&self, stick: &StickId) {
        self.queue.push(ApiMessage::SetStick(stick.clone()));
    }
}
