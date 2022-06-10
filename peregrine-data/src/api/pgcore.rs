use crate::core::channel::Channel;
use crate::core::version::VersionMetadata;
use crate::metric::metricreporter::MetricCollector;
use crate::core::{ Viewport };
use crate::request::core::manager::RequestManager;
use crate::request::messages::metricreq::MetricReport;
use crate::api::PeregrineIntegration;
use crate::train::main::railway::Railway;
use commander::PromiseFuture;
use peregrine_dauphin_queue::{ PgDauphinQueue };
use peregrine_message::PeregrineMessage;
use peregrine_toolkit::{lock};
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit_async::sync::needed::Needed;
use std::sync::{ Arc, Mutex };
use crate::{AllBackends, Assets, Commander, CountingPromise, PgCommander, PgDauphin};
use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::api::AgentStore;
use crate::core::{ StickId };
use crate::train::graphics::Graphics;
use crate::util::message::DataMessage;
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
    pub answer_allocator: Arc<Mutex<AnswerAllocator>>,
    pub messages: MessageSender,
    pub metrics: MetricCollector,
    pub dauphin_queue: PgDauphinQueue,
    pub dauphin: PgDauphin,
    pub commander: PgCommander,
    pub all_backends: AllBackends,
    pub manager: RequestManager,
    pub booted: CountingPromise,
    pub queue: PeregrineApiQueue,
    pub identity: Arc<Mutex<u64>>,
    pub(crate) graphics: Graphics,
    pub integration: Arc<Mutex<Box<dyn PeregrineIntegration>>>,
    pub assets: Arc<Mutex<Assets>>,
    pub version: VersionMetadata,
    pub redraw_needed: Needed
}

#[derive(Clone)]
pub struct PeregrineCore {
    pub base: PeregrineCoreBase,
    pub agent_store: AgentStore,
    pub train_set: Railway, // XXX into AgentStore
    pub viewport: Viewport,
    pub switches: Switches,
}

impl PeregrineCore {
    pub fn new<M,F>(integration: Box<dyn PeregrineIntegration>, commander: M, messages: F, queue: &PeregrineApiQueue, redraw_needed: &Needed) -> Result<PeregrineCore,DataMessage> 
                where M: Commander + 'static, F: FnMut(DataMessage) + 'static + Send {
        let integration = Arc::new(Mutex::new(integration));
        let graphics = Graphics::new(&integration);
        let commander = PgCommander::new(Box::new(commander));
        let metrics = MetricCollector::new(&commander);
        let messages = MessageSender::new(messages);
        let dauphin_queue = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&dauphin_queue).map_err(|e| DataMessage::DauphinIntegrationError(format!("could not create: {}",e)))?;
        let version = VersionMetadata::new();
        let manager = RequestManager::new(lock!(integration).channel(),&commander,&messages,&version);
        let all_backends = AllBackends::new(&manager,&metrics,&messages);
        let booted = CountingPromise::new();
        let base = PeregrineCoreBase {
            answer_allocator: Arc::new(Mutex::new(AnswerAllocator::new())),
            metrics,
            booted,
            commander,
            dauphin,
            dauphin_queue,
            manager,
            messages,
            all_backends,
            graphics,
            integration,
            queue: queue.clone(),
            identity: Arc::new(Mutex::new(0)),
            assets: Arc::new(Mutex::new(Assets::empty())),
            version,
            redraw_needed: redraw_needed.clone()
        };
        let agent_store = AgentStore::new(&base);
        let train_set = Railway::new(&base,&agent_store.lane_store,queue.visual_blocker());
        Ok(PeregrineCore {
            base,
            agent_store,
            train_set,
            viewport: Viewport::empty(),
            switches: Switches::new()
        })
    }

    pub fn dauphin_ready(&mut self) {
        self.base.manager.add_receiver(Box::new(self.base.dauphin.clone()));
    }

    pub fn application_ready(&mut self) {
        self.base.queue.clone().run(self);
        self.base.queue.push(ApiMessage::Ready);
    }

    pub fn bootstrap(&mut self, identity: u64, channel: Channel) {
        self.base.metrics.bootstrap(&channel,identity,&self.base.manager);
        self.base.queue.push(ApiMessage::Bootstrap(identity,channel));
    }

    pub fn transition_complete(&self) {
        self.base.queue.push(ApiMessage::TransitionComplete);
    }

    pub async fn jump(&self, location: &str) -> Option<(StickId,f64,f64)> {
        let p = PromiseFuture::new();
        self.base.queue.push(ApiMessage::Jump(location.to_string(),p.clone()));
        p.await
    }

    pub fn set_switch(&self, path: &[&str]) {
        self.base.queue.push(ApiMessage::SetSwitch(path.iter().map(|x| x.to_string()).collect()));
    }

    pub fn clear_switch(&self, path: &[&str]) {
        self.base.queue.push(ApiMessage::ClearSwitch(path.iter().map(|x| x.to_string()).collect()));
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        self.base.queue.push(ApiMessage::RadioSwitch(path.iter().map(|x| x.to_string()).collect(),yn));
    }

    pub fn invalidate(&self) {
        self.base.queue.push(ApiMessage::Invalidate);
    }

    pub fn set_position(&self, pos: f64) {
        self.base.queue.push(ApiMessage::SetPosition(pos));
    }

    pub fn set_bp_per_screen(&self, scale: f64) {
        self.base.queue.push(ApiMessage::SetBpPerScreen(scale));
    }

    pub fn set_min_px_per_carriage(&self, px: u32) {
        self.base.queue.push(ApiMessage::SetMinPxPerCarriage(px));
    }

    pub fn set_stick(&self, stick: &StickId) {
        self.base.queue.push(ApiMessage::SetStick(stick.clone()));
    }

    pub fn ping_trains(&self) {
        self.base.queue.push(ApiMessage::PingTrains);
    }

    pub fn report_message(&self, channel: &Channel, message: &(dyn PeregrineMessage + 'static)) {
        self.base.queue.push(ApiMessage::ReportMetric(channel.clone(),MetricReport::new_from_error_message(&self.base,message)));
    }

    pub fn general_metric(&self, name: &str, tags: Vec<(String,String)>, values: Vec<(String,f64)>) {
        self.base.queue.push(ApiMessage::GeneralMetric(name.to_string(),tags,values))
    }

    pub fn set_sketchy(&self, yn: bool) {
        self.base.queue.push(ApiMessage::Sketchy(yn));
    }
}
