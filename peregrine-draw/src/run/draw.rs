use crate::{integration::pgchannel::PgChannel, shape::core::stage::ReadStageAxis};
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
use super::frame::run_animations;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext, Element };
use crate::train::GlTrainSet;
use super::dom::PeregrineDom;
use crate::shape::core::stage::{ Stage, ReadStage };
use crate::webgl::global::WebGlGlobal;
use commander::{ Lock, LockGuard, cdr_lock, CommanderStream };
use peregrine_data::{ Channel, Track, StickId };
use crate::util::pgblackbox::setup_blackbox;

// TODO async/sync versions

pub trait PeregrineDrawApi {
    fn set_message_reporter<F>(&mut self,callback: F) where F: FnMut(Message) + 'static;
    fn setup_blackbox(&self, url: &str) -> Result<(),Message>;
    fn x(&self) -> Result<f64,Message>;
    fn y(&self) -> Result<f64,Message>;
    fn size(&self) -> Result<(f64,f64),Message>;
    fn bp_per_screen(&self) -> Result<f64,Message>;
    fn set_x(&mut self, x: f64);
    fn set_y(&mut self, y: f64);
    fn set_size(&mut self, x: f64, y: f64);
    fn set_bp_per_screen(&mut self, z: f64);
    fn bootstrap(&self, channel: Channel);
    fn add_track(&self, track: Track);
    fn remove_track(&self, track: Track);
    fn set_stick(&self, stick: &StickId);
}

#[derive(Clone)]
pub struct PeregrineDraw {
    messages: Arc<Mutex<Option<Box<dyn FnMut(Message)>>>>,
    lock: Lock,
    commander: PgCommanderWeb,
    data_api: PeregrineCore,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>
}

pub struct LockedPeregrineDraw<'t> {
    pub commander: &'t mut PgCommanderWeb,
    pub data_api: &'t mut PeregrineCore,
    pub trainset: &'t mut GlTrainSet,
    pub webgl: &'t mut Arc<Mutex<WebGlGlobal>>,
    pub stage: &'t mut Arc<Mutex<Stage>>,
    #[allow(unused)] // it's the drop we care about
    guard: LockGuard<'t>
}

impl PeregrineDraw {
    pub(crate) async fn lock<'t>(&'t mut self) -> LockedPeregrineDraw<'t> {
        let guard = cdr_lock(&self.lock).await;
        LockedPeregrineDraw{ 
            commander: &mut self.commander,
            data_api: &mut self.data_api,
            trainset: &mut self.trainset,
            webgl: &mut self.webgl,
            stage: &mut self.stage,
            guard
        }
    }
    
    pub fn commander(&self) -> PgCommanderWeb { self.commander.clone() } // XXX
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
impl PeregrineDraw {
    pub fn new(config: PeregrineConfig, dom: PeregrineDom) -> Result<PeregrineDraw,Message> {
        // XXX change commander init to allow message init to move to head
        let commander = PgCommanderWeb::new(&dom)?;
        commander.start();
        let messages = Arc::new(Mutex::new(None));
        let message_sender = setup_message_sending_task(&commander, messages.clone());
        let commander_id = commander.identity();
        message_register_default(commander_id);
        message_register_callback(Some(commander_id),move |message| {
            message_sender.add(message);            
        });
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&dom)?));
        let stage = Arc::new(Mutex::new(Stage::new()));
        let trainset = GlTrainSet::new(&config,&stage.lock().unwrap())?;
        let integration = Box::new(PgIntegration::new(PgChannel::new(),trainset.clone(),webgl.clone()));
        let mut core = PeregrineCore::new(integration,commander.clone(),move |e| {
            routed_message(Some(commander_id),Message::DataError(e))
        }).map_err(|e| Message::DataError(e))?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        core.application_ready();
        let mut out = PeregrineDraw {
            lock: commander.make_lock(),
            messages,
            data_api: core.clone(), commander, trainset, stage,  webgl
        };
        out.setup()?;
        Ok(out)
    }
    
    fn setup(&mut self) -> Result<(),Message> {
        run_animations(self);
        Ok(())
    }

    fn read_stage(&self) -> ReadStage { self.stage.lock().unwrap().read_stage() }
}

impl PeregrineDrawApi for PeregrineDraw {
    fn bootstrap(&self, channel: Channel) {
        self.data_api.bootstrap(channel)
    }

    fn set_message_reporter<F>(&mut self, callback: F) where F: FnMut(Message) + 'static {
        *self.messages.lock().unwrap() = Some(Box::new(callback));
    }

    fn setup_blackbox(&self, url: &str) -> Result<(),Message> {
        setup_blackbox(&self.commander,url);
        Ok(())
    }

    fn x(&self) -> Result<f64,Message> { self.stage.lock().unwrap().x().position() }
    fn y(&self) -> Result<f64,Message> { self.stage.lock().unwrap().y().position() }
    fn size(&self) -> Result<(f64,f64),Message> { 
        Ok((
            self.stage.lock().unwrap().y().position()?,
            self.stage.lock().unwrap().y().position()?
        ))
    }
    fn bp_per_screen(&self) -> Result<f64,Message> { self.stage.lock().unwrap().x().bp_per_screen() }
    
    fn set_x(&mut self, x: f64) {
        self.data_api.set_position(x);
        self.stage.lock().unwrap().x_mut().set_position(x);
    }

    fn set_y(&mut self, y: f64) {
        self.stage.lock().unwrap().y_mut().set_position(y);
    }

    fn set_size(&mut self, x: f64, y: f64) {
        self.stage.lock().unwrap().x_mut().set_size(x);
        self.stage.lock().unwrap().y_mut().set_size(y);
    }

    fn set_bp_per_screen(&mut self, z: f64) {
        self.data_api.set_bp_per_screen(z);
        self.stage.lock().unwrap().x_mut().set_bp_per_screen(z);
    }

    fn add_track(&self, track: Track) {
        self.data_api.add_track(track);
    }

    fn remove_track(&self, track: Track) {
        self.data_api.remove_track(track);
    }

    fn set_stick(&self, stick: &StickId) {
        self.data_api.set_stick(stick)
    }
}
// TODO redraw on track change etc.
