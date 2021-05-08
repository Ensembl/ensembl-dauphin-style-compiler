use crate::core::{ Viewport };
use crate::train::{ TrainSet };
use crate::api::PeregrineIntegration;
use peregrine_dauphin_queue::{ PgDauphinQueue };
use crate::request::channel::Channel;
use std::sync::{ Arc, Mutex };
use crate::{ 
    PgCommander, PgDauphin, ProgramLoader, RequestManager, StickStore, StickAuthorityStore, Commander,
    CountingPromise, LaneProgramLookup, LaneStore, DataStore
};
use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::api::AgentStore;
use crate::core::{ Focus, StickId };
use crate::util::message::DataMessage;
use peregrine_message::Instigator;
use crate::switch::switch::Switches;

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
    pub queue: PeregrineApiQueue,
}

#[derive(Clone)]
pub struct PeregrineCore {
    pub base: PeregrineCoreBase,
    pub agent_store: AgentStore,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub train_set: TrainSet,
    pub viewport: Viewport,
    pub switches: Switches,
}

impl PeregrineCore {
    pub fn new<M,F>(integration: Box<dyn PeregrineIntegration>, commander: M, messages: F) -> Result<PeregrineCore,DataMessage> 
                where M: Commander + 'static, F: FnMut(DataMessage) + 'static + Send {
        let mut agent_store = AgentStore::new();
        let messages = MessageSender::new(messages);
        let dauphin_queue = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&dauphin_queue).map_err(|e| DataMessage::DauphinIntegrationError(format!("could not create: {}",e)))?;
        let commander = PgCommander::new(Box::new(commander));
        let manager = RequestManager::new(integration.channel(),&commander,&messages);
        let booted = CountingPromise::new();
        let base = PeregrineCoreBase {
            booted,
            commander,
            dauphin,
            dauphin_queue,
            manager,
            messages,
            queue: PeregrineApiQueue::new(),
        };
        agent_store.set_data_store(DataStore::new(32,&base,&agent_store));
        agent_store.set_program_loader(ProgramLoader::new(&base));
        agent_store.set_stick_authority_store(StickAuthorityStore::new(&base,&agent_store));
        agent_store.set_stick_store(StickStore::new(&base,&agent_store));
        agent_store.set_lane_store(LaneStore::new(128,&base,&agent_store));
        agent_store.set_lane_program_lookup(LaneProgramLookup::new());
        let train_set = TrainSet::new(&base);
        if !agent_store.ready() {
            return Err(DataMessage::CodeInvariantFailed(format!("dependency injection failed")));
        }
        Ok(PeregrineCore {
            base,
            agent_store,
            integration: Arc::new(Mutex::new(integration)),
            train_set,
            viewport: Viewport::empty(),
            switches: Switches::new()
        })
    }

    pub fn dauphin_ready(&mut self) {
        self.base.manager.add_receiver(Box::new(self.base.dauphin.clone()));
    }

    pub fn application_ready(&mut self) {
        let instigator = Instigator::new();
        self.base.queue.clone().run(self);
        self.base.queue.push(ApiMessage::Ready,instigator);
    }

    pub fn bootstrap(&self, channel: Channel) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::Bootstrap(channel),instigator.clone());
        instigator
    }

    /* from api */
    pub fn ready(&self, mut core: PeregrineCore) {
        let instigator = Instigator::new();
        self.base.queue.run(&mut core);
        self.base.queue.push(ApiMessage::Ready,instigator);
    }

    /* called after someprograms to refresh state in-case tracks appeared */
    pub(crate) fn regenerate_track_config(&self, instigator: Instigator<DataMessage>) {
        self.base.queue.push(ApiMessage::RegeneraateTrackConfig,instigator);
    }

    pub fn backend_bootstrap(&self, channel: Channel) {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::Bootstrap(channel),instigator);
    }

    pub fn transition_complete(&self) {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::TransitionComplete,instigator);
    }

    pub fn set_switch(&self, path: &[&str]) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::SetSwitch(path.iter().map(|x| x.to_string()).collect()),instigator.clone());
        instigator
    }

    pub fn clear_switch(&self, path: &[&str]) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::ClearSwitch(path.iter().map(|x| x.to_string()).collect()),instigator.clone());
        instigator
    }

    pub fn set_position(&self, pos: f64) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::SetPosition(pos),instigator.clone());
        instigator
    }

    pub fn set_bp_per_screen(&self, scale: f64) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::SetBpPerScreen(scale),instigator.clone());
        instigator
    }

    pub fn set_focus(&self, focus: &Focus) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::SetFocus(focus.clone()),instigator.clone());
        instigator
    }

    pub fn set_stick(&self, stick: &StickId) -> Instigator<DataMessage> {
        let instigator = Instigator::new();
        self.base.queue.push(ApiMessage::SetStick(stick.clone()),instigator.clone());
        instigator
    }
}
