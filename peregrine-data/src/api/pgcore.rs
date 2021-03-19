use anyhow::bail;
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
use crate::api::AgentStore;
use crate::core::{ Track, Focus, StickId };
use crate::util::message::DataMessage;
use super::agentstore;

#[derive(Clone)]
pub struct MessageSender(Arc<Mutex<Box<dyn FnMut(DataMessage) + 'static + Send>>>);

impl MessageSender {
    pub(crate) fn new<F>(cb :F) -> MessageSender where F: FnMut(DataMessage) + 'static + Send {
        MessageSender(Arc::new(Mutex::new(Box::new(cb))))
    }

    pub(crate) fn send(&self,message: DataMessage) {
        (self.0.lock().unwrap())(message);
    }
}

#[derive(Clone)]
pub struct PeregrineCoreBase {
    pub messages: MessageSender,
    pub dauphin_queue: PgDauphinQueue,
    pub dauphin: PgDauphin,
    pub commander: PgCommander,
    pub manager: RequestManager,
    pub booted: CountingPromise,
}

#[derive(Clone)]
pub struct PeregrineCore {
    pub base: PeregrineCoreBase,
    pub agent_store: AgentStore,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub train_set: TrainSet,
    pub viewport: Viewport,
    pub queue: PeregrineApiQueue,
}

impl PeregrineCore {
    pub fn new<M,F>(integration: Box<dyn PeregrineIntegration>, commander: M, messages: F) -> anyhow::Result<PeregrineCore> 
                where M: Commander + 'static, F: FnMut(DataMessage) + 'static + Send {
        let mut agent_store = AgentStore::new();
        let messages = MessageSender(Arc::new(Mutex::new(Box::new(messages))));
        let dauphin_queue = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&dauphin_queue)?;
        let commander = PgCommander::new(Box::new(commander));
        let manager = RequestManager::new(integration.channel(),&commander,&messages);
        let booted = CountingPromise::new();
        let base = PeregrineCoreBase {
            booted,
            commander,
            dauphin,
            dauphin_queue,
            manager,
            messages
        };
        agent_store.set_data_store(DataStore::new(32,&base,&agent_store));
        agent_store.set_program_loader(ProgramLoader::new(&base,&agent_store));
        agent_store.set_stick_authority_store(StickAuthorityStore::new(&base,&agent_store));
        agent_store.set_stick_store(StickStore::new(&base,&agent_store));
        agent_store.set_panel_store(PanelStore::new(128,&base,&agent_store));
        agent_store.set_panel_program_store(PanelProgramStore::new());
        agent_store.set_panel_run_store(PanelRunStore::new(32,&base,&agent_store));
        let train_set = TrainSet::new(&base);
        if !agent_store.ready() {
            bail!("dependency injection failed");
        }
        Ok(PeregrineCore {
            base,
            agent_store,
            integration: Arc::new(Mutex::new(integration)),
            train_set,
            viewport: Viewport::empty(),
            queue: PeregrineApiQueue::new(),
        })
    }

    pub fn dauphin_ready(&mut self) {
        self.base.manager.add_receiver(Box::new(self.base.dauphin.clone()));
    }

    pub fn application_ready(&mut self) {
        self.queue.clone().run(self);
        self.queue.push(ApiMessage::Ready);
    }

    pub fn bootstrap(&self, channel: Channel) {
        bootstrap(&self.base,&self.agent_store,channel)
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

    pub fn set_bp_per_screen(&self, scale: f64) {
        self.queue.push(ApiMessage::SetBpPerScreen(scale));
    }

    pub fn set_focus(&self, focus: &Focus) {
        self.queue.push(ApiMessage::SetFocus(focus.clone()));
    }

    pub fn set_stick(&self, stick: &StickId) {
        self.queue.push(ApiMessage::SetStick(stick.clone()));
    }
}
