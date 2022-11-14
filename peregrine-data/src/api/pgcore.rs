use crate::core::channel::channelregistry::{ChannelRegistryBuilder, ChannelRegistry};
use crate::core::version::VersionMetadata;
use crate::metric::metricreporter::MetricCollector;
use crate::core::{ Viewport };
use crate::request::core::manager::{RequestManager, LowLevelRequestManager};
use crate::request::core::sidecars::RequestSidecars;
use crate::request::minirequests::metricreq::MetricReport;
use crate::api::PeregrineIntegration;
use crate::train::main::railway::Railway;
use commander::PromiseFuture;
use peregrine_dauphin_queue::{ PgDauphinQueue };
use peregrine_message::PeregrineMessage;
use peregrine_toolkit::eachorevery::eoestruct::{StructValue};
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit::puzzle::AnswerAllocator;
use peregrine_toolkit_async::sync::needed::Needed;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };
use crate::{AllBackends, Assets, Commander, CountingPromise, PgCommander, PgDauphin, BackendNamespace, ChannelIntegration, SettingMode };
use crate::api::PeregrineApiQueue;
use crate::api::queue::ApiMessage;
use crate::api::AgentStore;
use crate::core::{ StickId };
use crate::train::graphics::Graphics;
use crate::util::message::DataMessage;
use crate::switch::switches::Switches;

#[derive(Clone)]
pub struct MessageSender(Arc<Mutex<Box<dyn FnMut(Error) + 'static + Send>>>);

impl MessageSender {
    pub(crate) fn new<F>(cb :F) -> MessageSender where F: FnMut(Error) + 'static + Send {
        MessageSender(Arc::new(Mutex::new(Box::new(cb))))
    }

    pub(crate) fn send(&self,message: Error) {
        lock!(self.0)(message);
    }
}

#[derive(Clone)]
pub struct PeregrineCoreBase {
    pub answer_allocator: Arc<Mutex<AnswerAllocator>>,
    pub messages: MessageSender,
    pub metrics: MetricCollector,
    pub channel_registry: ChannelRegistry,
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
    pub redraw_needed: Needed,
    pub shutdown: OneShot
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
    pub fn new<M,F>(integration: Box<dyn PeregrineIntegration>, commander: M, messages: F, queue: &PeregrineApiQueue, redraw_needed: &Needed, mut channel_integrations: Vec<Rc<dyn ChannelIntegration>>) -> Result<PeregrineCore,DataMessage> 
                where M: Commander + 'static, F: FnMut(Error) + 'static + Send {
        let shutdown = OneShot::new();
        let integration = Arc::new(Mutex::new(integration));
        let graphics = Graphics::new(&integration);
        let commander = PgCommander::new(Box::new(commander));
        let metrics = MetricCollector::new(&commander,&shutdown);
        let messages = MessageSender::new(messages);
        let dauphin_queue = PgDauphinQueue::new(&shutdown);
        let booted = CountingPromise::new();
        let mut channel_registry = ChannelRegistryBuilder::new(&booted);
        for itn in channel_integrations.drain(..) {
            channel_registry.add(itn);
        }
        let channel_registry = channel_registry.build();
        let dauphin = PgDauphin::new(&dauphin_queue,&channel_registry,&booted).map_err(|e| DataMessage::XXXTransitional(Error::fatal(&format!("could not create: {}",e))))?;
        let mut switches = Switches::new(&dauphin);
        let version = VersionMetadata::new();
        let sidecars = RequestSidecars::new(&dauphin,&switches,&queue);
        let low_manager = LowLevelRequestManager::new(&sidecars,&commander,&shutdown,&messages,&version);
        let manager = RequestManager::new(&low_manager,&channel_registry);
        let all_backends = AllBackends::new(&manager,&metrics);
        switches.set_all_backends(&all_backends);
        dauphin.set_all_backends(&all_backends);
        let base = PeregrineCoreBase {
            answer_allocator: Arc::new(Mutex::new(AnswerAllocator::new())),
            channel_registry,
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
            redraw_needed: redraw_needed.clone(),
            shutdown
        };
        let agent_store = AgentStore::new(&base);
        base.channel_registry.run_boot_loop(&base);
        
        let train_set = Railway::new(&base,&agent_store.lane_store,queue.visual_blocker());
        Ok(PeregrineCore {
            base,
            agent_store,
            train_set,
            viewport: Viewport::empty(),
            switches: switches.clone()
        })
    }

    pub(crate) fn shutdown(&mut self) -> &OneShot { &self.base.shutdown }

    pub fn add_backend(&mut self, backend: &str) {
        self.base.queue.push(ApiMessage::AddBackend(backend.to_string()));
    }

    pub fn application_ready(&mut self) {
        self.base.queue.clone().run(self);
    }

    pub fn transition_complete(&self) {
        self.base.queue.push(ApiMessage::TransitionComplete);
    }

    pub async fn jump(&self, location: &str) -> Option<(StickId,f64,f64)> {
        let p = PromiseFuture::new();
        self.base.queue.push(ApiMessage::Jump(location.to_string(),p.clone()));
        p.await
    }

    pub fn switch(&self, path: &[&str], value: StructValue) {
        self.base.queue.push(ApiMessage::Switch(path.iter().map(|x| x.to_string()).collect(),value));
    }

    pub fn update_switch(&self, path: &[&str], value: SettingMode) {
        self.base.queue.push(ApiMessage::UpdateSwitch(path.iter().map(|x| x.to_string()).collect(),value));
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

    pub fn report_message(&self, channel: &BackendNamespace, message: &(dyn PeregrineMessage + 'static)) {
        self.base.queue.push(ApiMessage::ReportMetric(channel.clone(),MetricReport::new_from_error_message(&self.base,message)));
    }

    pub fn general_metric(&self, name: &str, tags: Vec<(String,String)>, values: Vec<(String,f64)>) {
        self.base.queue.push(ApiMessage::GeneralMetric(name.to_string(),tags,values))
    }

    pub fn set_sketchy(&self, yn: bool) {
        self.base.queue.push(ApiMessage::Sketchy(yn));
    }
}
