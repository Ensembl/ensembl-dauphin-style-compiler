use crate::input::Input;
use crate::{integration::pgchannel::PgChannel };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use std::sync::{ Mutex, Arc };
use crate::util::message::{ Message, message_register_callback, routed_message, message_register_default };

use peregrine_data::{ 
    Commander,
    PeregrineCore
};
use peregrine_dauphin::peregrine_dauphin;
use peregrine_message::Instigator;
use super::report::Report;
use super::{PgPeregrineConfig, globalconfig::CreatedPeregrineConfigs};
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use crate::train::GlTrainSet;
use super::dom::PeregrineDom;
use crate::stage::stage::{ Stage };
use crate::webgl::global::WebGlGlobal;
use commander::{CommanderStream, Lock, LockGuard, cdr_lock};
use peregrine_data::{ Channel, StickId, DataMessage, Viewport };
use crate::shape::core::spectremanager::SpectreManager;

fn data_inst(inst: &mut Instigator<Message>, inst_data: Instigator<DataMessage>) {
    inst.merge(inst_data,|e| Message::DataError(e));
}

fn draw_inst(inst: &mut Instigator<Message>, result: Result<(),Message>) {
    if let Err(e) = result {
        inst.error(e);
    }
}

#[derive(Clone)]
pub struct Target {
    viewport: Viewport,
    size: Option<(u32,u32)>,
    y: f64
}

impl Target {
    pub fn new() -> Target {
        Target {
            viewport: Viewport::empty(),
            size: None,
            y: 0.
        }
    }

    pub fn x(&self) -> Result<Option<f64>,Message> {
        if !self.viewport.ready() { return Ok(None); }
        Ok(Some(self.viewport.position().map_err(|e| Message::DataError(e))?))
    }

    pub fn bp_per_screen(&self) -> Result<Option<f64>,Message> {
        if !self.viewport.ready() { return Ok(None); }
        Ok(Some(self.viewport.bp_per_screen().map_err(|e| Message::DataError(e))?))
    }

    pub fn size(&self) -> Option<&(u32,u32)> { self.size.as_ref() }
    pub fn y(&self) -> f64 { self.y }
}

#[derive(Clone)]
pub struct PeregrineInnerAPI {
    config: Arc<PgPeregrineConfig>,
    messages: Arc<Mutex<Option<Box<dyn FnMut(Message)>>>>,
    message_sender: CommanderStream<Message>,
    lock: Lock,
    commander: PgCommanderWeb,
    data_api: PeregrineCore,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    dom: PeregrineDom,
    spectre_manager: SpectreManager,
    input: Input,
    report: Report
}

pub struct LockedPeregrineInnerAPI<'t> {
    pub commander: &'t mut PgCommanderWeb,
    pub data_api: &'t mut PeregrineCore,
    pub trainset: &'t mut GlTrainSet,
    pub webgl: &'t mut Arc<Mutex<WebGlGlobal>>,
    pub stage: &'t mut Arc<Mutex<Stage>>,
    pub message_sender: &'t mut CommanderStream<Message>,
    pub dom: &'t mut PeregrineDom,
    pub(crate) spectre_manager: &'t mut SpectreManager,
    pub report: &'t Report,
    pub input: &'t Input,
    #[allow(unused)] // it's the drop we care about
    guard: LockGuard<'t>
}

/* There's a separate message sending task so that there's no problems with recursive message sending.
 * their_callback is synchronous so cannot cause this loop to run recursively even if it issues API commands.
 */
async fn message_sending_task(our_queue: CommanderStream<Message>, their_callback: Arc<Mutex<Option<Box<dyn FnMut(Message)>>>>) -> Result<(),Message> {
    loop {
        let message = our_queue.get().await;
        let mut callback = their_callback.lock().unwrap();
        if let Some(callback) = callback.as_mut() {
            callback(message);
        }
        drop(callback);
    }
}

fn setup_message_sending_task(commander: &PgCommanderWeb, their_callback: Arc<Mutex<Option<Box<dyn FnMut(Message)>>>>) -> CommanderStream<Message> {
    let stream = CommanderStream::new();
    commander.add("message-sender",10,None,None,Box::pin(message_sending_task(stream.clone(),their_callback)));
    // TODO failure handling
    stream
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
            guard
        }
    }
    
    pub fn commander(&self) -> PgCommanderWeb { self.commander.clone() } // XXX

    pub(super) fn new(config: &CreatedPeregrineConfigs, dom: &PeregrineDom, commander: &PgCommanderWeb) -> Result<PeregrineInnerAPI,Message> {
        let commander = commander.clone();
        // XXX change commander init to allow message init to move to head
        let messages = Arc::new(Mutex::new(None));
        let message_sender = setup_message_sending_task(&commander, messages.clone());
        let commander_id = commander.identity();
        message_register_default(commander_id);
        let message_sender2 = message_sender.clone();
        message_register_callback(Some(commander_id),move |message| {
            message_sender2.add(message);
        });
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&dom,&config.draw)?));
        let stage = Arc::new(Mutex::new(Stage::new()));
        let trainset = GlTrainSet::new(&config.draw,&stage.lock().unwrap())?;
        let report = Report::new(&config.draw,&message_sender)?;
        let integration = Box::new(PgIntegration::new(PgChannel::new(),trainset.clone(),webgl.clone(),&stage,&report));
        let mut core = PeregrineCore::new(integration,commander.clone(),move |e| {
            routed_message(Some(commander_id),Message::DataError(e))
        }).map_err(|e| Message::DataError(e))?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        let redraw_needed = stage.lock().unwrap().redraw_needed();
        let mut input = Input::new();
        report.run(&commander);
        core.application_ready();
        message_sender.add(Message::Ready);
        let out = PeregrineInnerAPI {
            config: config.draw.clone(),
            lock: commander.make_lock(),
            messages,
            message_sender: message_sender.clone(),
            data_api: core.clone(),
            commander: commander.clone(),
            trainset, stage, webgl,
            dom: dom.clone(),
            spectre_manager: SpectreManager::new(&config.draw,&redraw_needed),
            input: input.clone(),
            report: report.clone()
        };
        input.set_api(dom,&config.draw,&out,&commander,&report)?;
        message_sender.add(Message::Ready);
        Ok(out)
    }

    pub(crate) fn spectres(&self) -> &SpectreManager { &self.spectre_manager }
    pub(crate) fn stage(&self) -> &Arc<Mutex<Stage>> { &self.stage }

    pub(crate) fn set_artificial(&self, name: &str, start: bool) {
        self.input.set_artificial(name,start);
    }

    pub(crate) fn set_switch(&self, path: &[&str], instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_switch(path));
    }

    pub(crate) fn clear_switch(&self, path: &[&str], instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.clear_switch(path));
    }

    pub(super) fn config(&self) -> &PgPeregrineConfig { &self.config }

    pub(super) fn bootstrap(&self, channel: Channel, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.bootstrap(channel));
        instigator.done();
    }

    pub(super) fn set_message_reporter(&mut self, callback: Box<dyn FnMut(Message) + 'static>) {
        *self.messages.lock().unwrap() = Some(callback);
        self.message_sender.add(Message::Ready);
    }
    
    pub(crate) fn set_x(&mut self, x: f64, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_position(x));
        instigator.done();
    }

    pub(super) fn set_y(&mut self, y: f64) {
        self.stage.lock().unwrap().y_mut().set_position(y);
    }

    pub(crate) fn set_bp_per_screen(&mut self, z: f64, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_bp_per_screen(z));
        instigator.done();
    }

    pub(super) fn goto(&mut self, centre: f64, scale: f64, instigator: &mut Instigator<Message>) {
        draw_inst(instigator,self.input.clone().goto(self,centre,scale));      
        instigator.done();
    }

    pub(super) fn set_stick(&self, stick: &StickId, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_stick(stick));
        instigator.done();
    }

    pub(crate) fn debug_action(&self, index: u8) {
        use crate::stage::axis::ReadStageAxis;
        console::log_1(&format!("received debug action {}",index).into());
        if index == 9 {
            let stage = self.stage.lock().unwrap();
            console::log_1(&format!("x {:?} bp_per_screen {:?}",stage.x().position(),stage.x().bp_per_screen()).into());
        }
    }
}
// TODO redraw on track change etc.
