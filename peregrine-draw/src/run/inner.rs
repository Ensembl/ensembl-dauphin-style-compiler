use crate::{integration::pgchannel::PgChannel };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use std::sync::{ Mutex, Arc };
use crate::util::message::{ Message, message_register_callback, routed_message, message_register_default };

use peregrine_data::{ 
    Commander,
    PeregrineCore,
    PeregrineConfig
};
use peregrine_dauphin::peregrine_dauphin;
use peregrine_message::Instigator;
use super::frame::run_animations;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use crate::train::GlTrainSet;
use super::dom::PeregrineDom;
use crate::stage::stage::{ Stage, Position };
use crate::webgl::global::WebGlGlobal;
use commander::{CommanderStream, Lock, LockGuard, cdr_lock};
use peregrine_data::{ Channel, Track, StickId, DataMessage };

#[cfg(blackbox)]
pub fn setup_blackbox(commander: &PgCommanderWeb, url: &str) {
    use crate::util::pgblackbox::setup_blackbox_real;

    setup_blackbox_real(commander,url);
}

#[cfg(not(blackbox))]
pub fn setup_blackbox(_commander: &PgCommanderWeb, _url: &str) {
}

fn data_inst(inst: &mut Instigator<Message>, inst_data: Instigator<DataMessage>) {
    inst.merge(inst_data,|e| Message::DataError(e));
}

#[derive(Clone)]
pub struct PeregrineInnerAPI {
    messages: Arc<Mutex<Option<Box<dyn FnMut(Message)>>>>,
    message_sender: CommanderStream<Message>,
    position_callbacks: Arc<Mutex<Option<Box<dyn FnMut(Option<Position>)>>>>,
    lock: Lock,
    commander: PgCommanderWeb,
    data_api: PeregrineCore,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>,
    position: Option<Position>,
    dom: PeregrineDom
}

pub struct LockedPeregrineInnerAPI<'t> {
    pub commander: &'t mut PgCommanderWeb,
    pub data_api: &'t mut PeregrineCore,
    pub trainset: &'t mut GlTrainSet,
    pub webgl: &'t mut Arc<Mutex<WebGlGlobal>>,
    pub stage: &'t mut Arc<Mutex<Stage>>,
    pub message_sender: &'t mut CommanderStream<Message>,
    pub dom: &'t mut PeregrineDom,
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
            guard
        }
    }
    
    pub fn commander(&self) -> PgCommanderWeb { self.commander.clone() } // XXX

    pub(super) fn new(config: PeregrineConfig, dom: PeregrineDom) -> Result<PeregrineInnerAPI,Message> {
        // XXX change commander init to allow message init to move to head
        let commander = PgCommanderWeb::new(&dom)?;
        commander.start();
        let messages = Arc::new(Mutex::new(None));
        let message_sender = setup_message_sending_task(&commander, messages.clone());
        let commander_id = commander.identity();
        message_register_default(commander_id);
        let message_sender2 = message_sender.clone();
        message_register_callback(Some(commander_id),move |message| {
            message_sender2.add(message);            
        });
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&dom)?));
        let stage = Arc::new(Mutex::new(Stage::new()));
        let trainset = GlTrainSet::new(&config,&stage.lock().unwrap())?;
        let integration = Box::new(PgIntegration::new(PgChannel::new(),trainset.clone(),webgl.clone()));
        let mut core = PeregrineCore::new(integration,commander.clone(),move |e| {
            routed_message(Some(commander_id),Message::DataError(e))
        }).map_err(|e| Message::DataError(e))?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        let dom2 = dom.clone();
        core.application_ready();
        let mut out = PeregrineInnerAPI {
            lock: commander.make_lock(),
            messages, message_sender,
            position_callbacks: Arc::new(Mutex::new(None)),
            data_api: core.clone(), commander, trainset, stage,  webgl,
            position: None,
            dom
        };
        out.setup(&dom2)?;
        Ok(out)
    }
    
    pub(super) fn xxx_set_callbacks<F>(&self, cb: F) where F: FnMut(Option<Position>) + 'static {
        *self.position_callbacks.lock().unwrap() = Some(Box::new(cb));
    }

    pub fn set_position(&self, position: Option<Position>) {
        if let Some(cb) = self.position_callbacks.lock().unwrap().as_mut() {
            cb(position);
        }
    }

    fn setup(&mut self, dom: &PeregrineDom) -> Result<(),Message> {
        run_animations(self,dom)?;
        Ok(())
    }

    pub(super) fn bootstrap(&self, channel: Channel, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.bootstrap(channel));
        instigator.done();
    }

    pub(super) fn set_message_reporter(&mut self, callback: Box<dyn FnMut(Message) + 'static>) {
        *self.messages.lock().unwrap() = Some(callback);
    }

    pub(super) fn setup_blackbox(&self, url: &str) -> Result<(),Message> {
        setup_blackbox(&self.commander,url);
        Ok(())
    }
    
    pub(super) fn set_x(&mut self, x: f64, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_position(x));
        self.stage.lock().unwrap().x_mut().set_position(x);
        instigator.done();
    }

    pub(super) fn set_y(&mut self, y: f64) {
        self.stage.lock().unwrap().y_mut().set_position(y);
    }

    pub(super) fn set_bp_per_screen(&mut self, z: f64, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_bp_per_screen(z));
        self.stage.lock().unwrap().x_mut().set_bp_per_screen(z);
        instigator.done();
    }

    pub(super) fn add_track(&self, track: Track, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.add_track(track));
        instigator.done();
    }

    pub(super) fn remove_track(&self, track: Track, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.remove_track(track));
        instigator.done();
    }

    pub(super) fn set_stick(&self, stick: &StickId, instigator: &mut Instigator<Message>) {
        data_inst(instigator,self.data_api.set_stick(stick));
        instigator.done();
    }
}
// TODO redraw on track change etc.
