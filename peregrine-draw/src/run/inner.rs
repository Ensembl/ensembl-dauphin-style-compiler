use crate::domcss::dom::PeregrineDom;
use crate::input::Input;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use std::rc::Rc;
use std::sync::{ Mutex, Arc };
use crate::util::message::{ Message, message_register_callback, routed_message, message_register_default };
use crate::input::translate::targetreporter::TargetReporter;
use js_sys::Date;
use peregrine_data::{Assets, Commander, PeregrineCore, PeregrineApiQueue, BackendNamespace, ChannelIntegration, DataMessage};
use peregrine_dauphin::peregrine_dauphin;
use peregrine_febe_javascript::JavascriptIntegration;
use peregrine_febe_network::NetworkChannel;
use peregrine_message::MessageKind;
use peregrine_toolkit::eachorevery::eoestruct::{StructValue};
use peregrine_toolkit::error::err_web_drop;
use peregrine_toolkit::log;
use peregrine_toolkit::plumbing::distributor::Distributor;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::blocker::Blocker;
use peregrine_toolkit_async::sync::needed::Needed;
use wasm_bindgen::JsValue;
use super::report::Report;
use super::sound::Sound;
use super::{PgPeregrineConfig, globalconfig::CreatedPeregrineConfigs};
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use crate::train::GlRailway;
use crate::stage::stage::{ Stage };
use crate::webgl::global::WebGlGlobal;
use commander::{CommanderStream, Lock, LockGuard, cdr_lock};
use peregrine_data::{ StickId };
use crate::shape::core::spectremanager::SpectreManager;
use peregrine_message::PeregrineMessage;
use js_sys::Math::random;

#[derive(Clone)]
pub struct PeregrineInnerAPI {
    config: Arc<PgPeregrineConfig>,
    messages: Distributor<Message>,
    message_sender: CommanderStream<Option<Message>>,
    lock: Lock,
    commander: PgCommanderWeb,
    data_api: PeregrineCore,
    trainset: GlRailway,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    dom: PeregrineDom,
    spectre_manager: SpectreManager,
    input: Input,
    report: Report,
    sound: Sound,
    assets: Assets,
    target_reporter: TargetReporter,
    jsapi: JavascriptIntegration
}

pub struct LockedPeregrineInnerAPI<'t> {
    pub commander: &'t mut PgCommanderWeb,
    pub data_api: &'t mut PeregrineCore,
    pub trainset: &'t mut GlRailway,
    pub webgl: &'t mut Arc<Mutex<WebGlGlobal>>,
    pub stage: &'t mut Arc<Mutex<Stage>>,
    pub message_sender: &'t mut CommanderStream<Option<Message>>,
    pub(crate) dom: &'t mut PeregrineDom,
    pub(crate) spectre_manager: &'t mut SpectreManager,
    pub report: &'t Report,
    pub input: &'t Input,
    pub sound: &'t mut Sound,
    pub assets: &'t Assets,
    #[allow(unused)] // it's the drop we care about
    guard: LockGuard<'t>
}

/* There's a separate message sending task so that there's no problems with recursive message sending.
 * their_callback is synchronous so cannot cause this loop to run recursively even if it issues API commands.
 */
async fn message_sending_task(our_queue: CommanderStream<Option<Message>>, distributor: Distributor<Message>) -> Result<(),Message> {
    while let Some(message) = our_queue.get().await {
        distributor.send(message);
    }
    Ok(())
}

fn setup_message_sending_task(commander: &PgCommanderWeb, distributor: Distributor<Message>, shutdown: &OneShot) -> CommanderStream<Option<Message>> {
    let stream = CommanderStream::new();
    let stream2 = stream.clone();
    shutdown.add(move || { stream2.add(None); });
    commander.add("message-sender",7,None,None,Box::pin(message_sending_task(stream.clone(),distributor)));
    // TODO failure handling
    stream
}

fn send_errors_to_backend(channel: &BackendNamespace, data_api: &PeregrineCore) -> impl FnMut(&Message) + 'static {
    let data_api = data_api.clone();
    let channel = channel.clone();
    move |message| {
        match message.kind() { 
            MessageKind::Error => {
                data_api.report_message(&channel,message);
            },
            _ => {}
        }
    }
}

// TODO redraw on change (? eh?)
// TODO end buffers
impl PeregrineInnerAPI {
    pub(crate) async fn lock<'t>(&'t mut self) -> LockedPeregrineInnerAPI<'t> {
        let guard = cdr_lock(&self.lock).await;
        LockedPeregrineInnerAPI{ 
            commander: &mut self.commander,
            data_api: &mut self.data_api,
            trainset: &mut self.trainset,
            webgl: &mut self.webgl,
            stage: &mut self.stage,
            message_sender: &mut self.message_sender,
            dom: &mut self.dom,
            spectre_manager: &mut self.spectre_manager,
            input: &mut self.input,
            report: &mut self.report,
            sound: &mut self.sound,
            assets: &mut self.assets,
            guard
        }
    }
    
    pub fn commander(&self) -> PgCommanderWeb { self.commander.clone() } // XXX

    pub(super) fn new(config: &CreatedPeregrineConfigs, dom: &PeregrineDom, commander: &PgCommanderWeb, backend: &str, queue_blocker: &Blocker, redraw_needed: &Needed) -> Result<PeregrineInnerAPI,Message> {
        let commander = commander.clone();
        // XXX change commander init to allow message init to move to head
        let mut messages = Distributor::new();
        let message_sender = setup_message_sending_task(&commander, messages.clone(),dom.shutdown());
        let commander_id = commander.identity();
        message_register_default(commander_id);
        let message_sender2 = message_sender.clone();
        message_register_callback(Some(commander_id),move |message| {
            message_sender2.add(Some(message));
        });
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&commander,&dom,&config.draw)?));
        let stage = Arc::new(Mutex::new(Stage::new(&redraw_needed)));
        let report = Report::new(&config.draw,&message_sender,&dom.shutdown())?;
        let target_reporter = TargetReporter::new(&commander,dom.shutdown(),&config.draw,&report)?;
        let mut input = Input::new(queue_blocker);
        let api_queue = PeregrineApiQueue::new(queue_blocker);
        let trainset = GlRailway::new(&api_queue,&commander,&config.draw,&stage.lock().unwrap())?;
        let integration = Box::new(PgIntegration::new(trainset.clone(),&input,webgl.clone(),&stage,&dom,&report));
        let assets = integration.assets().clone();
        let sound = Sound::new(&config.draw,&commander,integration.assets(),&mut messages,dom.shutdown())?;
        let jsapi = JavascriptIntegration::new();
        let channel_integrations : Vec<Rc<dyn ChannelIntegration>> = vec![
            Rc::new(jsapi.clone()),
            Rc::new(NetworkChannel::new()),
        ];
        let mut core = PeregrineCore::new(integration,commander.clone(),move |e| {
            routed_message(Some(commander_id),Message::DataError(DataMessage::XXXTransitional(e)))
        },&api_queue,&redraw_needed,channel_integrations).map_err(|e| Message::DataError(e))?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        core.add_backend(backend);
        report.run(&commander,&dom.shutdown());
        core.application_ready();
        message_sender.add(Some(Message::Ready));
        let out = PeregrineInnerAPI {
            config: config.draw.clone(),
            lock: commander.make_lock(),
            messages,
            message_sender: message_sender.clone(),
            data_api: core.clone(),
            commander: commander.clone(),
            trainset, stage, webgl,
            dom: dom.clone(),
            spectre_manager: SpectreManager::new(&commander,&config.draw,&redraw_needed),
            input: input.clone(),
            sound: sound.clone(),
            report: report.clone(),
            assets,
            target_reporter: target_reporter.clone(),
            jsapi
        };
        input.set_api(dom,&config.draw,&out,&commander,&target_reporter,&out.webgl)?;
        message_sender.add(Some(Message::Ready));
        dom.shutdown().add(move || {
            api_queue.shutdown();
        });
        Ok(out)
    }

    pub(crate) fn spectres(&self) -> &SpectreManager { &self.spectre_manager }
    pub(crate) fn stage(&self) -> &Arc<Mutex<Stage>> { &self.stage }

    pub(crate) fn set_artificial(&self, name: &str, start: bool) {
        self.input.set_artificial(name,start);
    }

    pub(crate) fn switch(&self, path: &[&str], value: StructValue) {
        self.data_api.switch(path,value);
    }

    pub(crate) fn set_sketchy(&self, yn: bool) {
        self.data_api.set_sketchy(yn);
    }

    pub(crate) fn radio_switch(&self, path: &[&str], yn: bool) {
        self.data_api.radio_switch(path,yn);
    }

    pub(super) fn config(&self) -> &PgPeregrineConfig { &self.config }

    pub(super) fn bootstrap(&mut self, channel: BackendNamespace) {
        let identity = (Date::now() + random()) as u64;
        self.messages.add(send_errors_to_backend(&channel,&self.data_api));
    }

    pub(super) fn set_message_reporter(&mut self, callback: Box<dyn FnMut(&Message) + 'static>) {
        self.messages.add(callback);
        self.message_sender.add(Some(Message::Ready));
    }

    pub(crate) fn invalidate(&mut self) {
        self.data_api.invalidate();
    }

    pub(crate) fn set_x(&mut self, x: f64) {
        self.data_api.set_position(x);
        self.target_reporter.set_x(x);
    }

    pub(super) fn set_y(&mut self, y: f64) {
        self.stage.lock().unwrap().y_mut().set_position(y);
    }

    pub(super) fn jump(&mut self, location: &str) {
        self.input.jump(&self.data_api,&self.commander,location);
    }

    pub(crate) fn set_bp_per_screen(&mut self, z: f64) {
        self.data_api.set_bp_per_screen(z);
        self.target_reporter.set_bp(z);
    }

    pub(super) fn goto(&mut self, centre: f64, scale: f64) {
        self.input.clone().goto(centre,scale);      
    }

    pub(crate) fn set_stick(&self, stick: &StickId) {
        self.stage.lock().unwrap().soon_stick(stick);
        self.data_api.set_stick(stick);
        self.target_reporter.set_stick(stick.get_id());
    }

    pub(crate) fn add_jsapi_channel(&mut self, name: &str, payload: JsValue) {
        err_web_drop(self.jsapi.add_channel(name,payload));
        self.data_api.add_backend(&format!("jsapi:{}",name));
    }

    pub(crate) fn debug_action(&mut self, index: u8) {
        use crate::stage::axis::ReadStageAxis;
        log!("received debug action {}",index);
        if index == 9 {
            let stage = self.stage.lock().unwrap();
            log!("x {:?} bp_per_screen {:?}",stage.x().position(),stage.x().bp_per_screen());
        } else if index == 8 {
            self.sound.play(None,"bell");
        }
    }
}
// TODO redraw on track change etc.
