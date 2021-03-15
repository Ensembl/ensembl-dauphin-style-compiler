use anyhow::{ self, anyhow as err };
use crate::{integration::pgchannel::PgChannel, shape::core::stage::ReadStageAxis};
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use std::sync::{ Mutex, Arc };
use crate::util::message::{ Message, message_register_callback, routed_message, message_register_default };

use crate::util::error::{ js_option };
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
use wasm_bindgen::JsCast;
use crate::shape::core::stage::{ Stage, ReadStage };
use crate::webgl::global::WebGlGlobal;
use commander::{ Lock, LockGuard, cdr_lock };
use peregrine_data::{ Channel, Track, StickId };

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

#[derive(Clone)]
pub struct PeregrineDraw {
    lock: Lock,
    commander: PgCommanderWeb,
    data_api: PeregrineCore,
    trainset: GlTrainSet,
    webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>
}

// TODO async/sync versions

pub trait PeregrineDrawApi {
    fn x(&self) -> anyhow::Result<f64>;
    fn y(&self) -> anyhow::Result<f64>;
    fn size(&self) -> anyhow::Result<(f64,f64)>;
    fn bp_per_screen(&self) -> anyhow::Result<f64>;
    fn set_x(&mut self, x: f64);
    fn set_y(&mut self, x: f64);
    fn set_size(&mut self, x: f64, y: f64);
    fn set_bp_per_screen(&mut self, z: f64);
    fn bootstrap(&self, channel: Channel);
    fn add_track(&self, track: Track);
    fn remove_track(&self, track: Track);
    fn set_stick(&self, stick: &StickId);
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

// TODO redraw on change (? eh?)
// TODO end buffers

impl PeregrineDraw {
    pub fn new<F>(config: PeregrineConfig, canvas: Element, messages: F) -> anyhow::Result<PeregrineDraw> where F: FnMut(Message) + 'static + Send {
        // XXX change commander init to allow message init to move to head
        let window = js_option(web_sys::window(),"cannot get window")?;
        let document = js_option(window.document(),"cannot get document")?;
        let html = js_option(document.body().clone(),"cannot get body")?;
        let commander = PgCommanderWeb::new(&html)?;
        commander.start();
        let commander_id = commander.identity();
        message_register_callback(Some(commander_id),messages);
        message_register_default(commander_id);
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| err!("cannot cast to canvas"))?;
        let context = canvas
            .get_context("webgl").map_err(|_| err!("cannot get webgl context"))?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().map_err(|_| err!("cannot get webgl context"))?;
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&document,&context)?));
        let stage = Arc::new(Mutex::new(Stage::new()));
        let trainset = GlTrainSet::new(&config,&stage.lock().unwrap())?;
        let integration = Box::new(PgIntegration::new(PgChannel::new(),trainset.clone(),webgl.clone()));
        let mut core = PeregrineCore::new(integration,commander.clone(),move |e| {
            routed_message(Some(commander_id),Message::DataError(e))
        })?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        core.application_ready();
        let mut out = PeregrineDraw {
            lock: commander.make_lock(),
            data_api: core.clone(), commander, trainset, stage,  webgl
        };
        out.setup()?;
        Ok(out)
    }
    
    fn setup(&mut self) -> anyhow::Result<()> {
        run_animations(self);
        Ok(())
    }

    fn read_stage(&self) -> ReadStage { self.stage.lock().unwrap().read_stage() }
}

impl PeregrineDrawApi for PeregrineDraw {
    fn bootstrap(&self, channel: Channel) {
        self.data_api.bootstrap(channel)
    }

    fn x(&self) -> anyhow::Result<f64> { self.stage.lock().unwrap().x().position() }
    fn y(&self) -> anyhow::Result<f64> { self.stage.lock().unwrap().y().position() }
    fn size(&self) -> anyhow::Result<(f64,f64)> { 
        Ok((
            self.stage.lock().unwrap().y().position()?,
            self.stage.lock().unwrap().y().position()?
        ))
    }
    fn bp_per_screen(&self) -> anyhow::Result<f64> { self.stage.lock().unwrap().x().bp_per_screen() }
    
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
