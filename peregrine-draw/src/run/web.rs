use anyhow::{ self, Context, anyhow as err };
use crate::integration::pgchannel::PgChannel;
use crate::integration::pgconsole::{ PgConsoleWeb };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use std::sync::{ Mutex, Arc };

#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use crate::util::error::{ js_option };
use peregrine_data::{ 
    Commander,
    PeregrineCore,
    PeregrineConfig
};
use peregrine_dauphin::peregrine_dauphin;
use super::frame::run_animations;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };
use crate::train::GlTrainSet;
use wasm_bindgen::JsCast;
use crate::shape::core::stage::{ Stage, ReadStage };
use crate::webgl::global::WebGlGlobal;

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

fn setup_commander() -> anyhow::Result<PgCommanderWeb> {
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let html = js_option(document.body().clone(),"cannot get body")?;
    let commander = PgCommanderWeb::new(&html)?;
    commander.start();
    Ok(commander)
}

// XXX not pub
#[derive(Clone)]
pub struct PeregrineWeb {
    pub commander: PgCommanderWeb,
    pub data_api: PeregrineCore,
    pub trainset: GlTrainSet,
    pub webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Arc<Mutex<Stage>>
}

impl PeregrineWeb {
    pub fn new() -> anyhow::Result<PeregrineWeb> {
        let commander = setup_commander().context("setting up commander")?;
        let console = PgConsoleWeb::new(30,30.);
        let mut config = PeregrineConfig::new();
        config.set_f64("animate.fade.slow",500.);
        config.set_f64("animate.fade.fast",100.);
        /* XXX separate out per canvase stuff */
        let window = js_option(web_sys::window(),"cannot get window")?;
        let document = js_option(window.document(),"cannot get document")?;
        // Nonsense
        let canvas = js_option(document.get_element_by_id("trainset"),"canvas gone AWOL")?;
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| err!("cannot cast to canvas"))?;
        let context = canvas
            .get_context("webgl").map_err(|_| err!("cannot get webgl context"))?
            .unwrap()
            .dyn_into::<WebGlRenderingContext>().map_err(|_| err!("cannot get webgl context"))?;
        // end of nonsense
        let webgl = Arc::new(Mutex::new(WebGlGlobal::new(&document,&context)?));
        let stage = Arc::new(Mutex::new(Stage::new()));
        let trainset = GlTrainSet::new(&config,&stage.lock().unwrap())?;
        let integration = Box::new(PgIntegration::new(PgChannel::new(console.clone()),trainset.clone(),webgl.clone()));
        let mut core = PeregrineCore::new(integration,commander.clone())?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&core);
        core.application_ready();
        let mut out = PeregrineWeb {
            data_api: core.clone(), commander, trainset, stage,  webgl
        };
        out.setup()?;
        Ok(out)
    }

    #[cfg(blackbox)]
    fn setup_blackbox(&self) {
        let mut ign = pgblackbox_setup();
        ign.set_url(&Url::parse("http://localhost:4040/blackbox/data").expect("bad blackbox url"));
        let ign2 = ign.clone();
        blackbox_enable("notice");
        blackbox_enable("warn");
        blackbox_enable("error");
        self.commander.add_task("blackbox",10,None,None,Box::pin(async move { ign2.sync_task().await?; Ok(()) }));
        blackbox_log("general","blackbox configured");
        console::log_1(&format!("blackbox configured").into());
    }

    #[cfg(not(blackbox))]
    fn setup_blackbox(&self) {
    }

    fn setup(&mut self) -> anyhow::Result<()> {
        self.setup_blackbox();
        run_animations(self);
        Ok(())
    }

    pub(crate) fn read_stage(&self) -> ReadStage { self.stage.lock().unwrap().read_stage() }

    // TODO redraw on change
    pub fn set_x_position(&mut self, x: f64) {
        self.data_api.set_position(x);
        self.stage.lock().unwrap().x_mut().set_position(x);
    }

    pub fn set_y_position(&mut self, x: f64) {
        self.stage.lock().unwrap().y_mut().set_position(x);
    }

    pub fn set_size(&mut self, x: f64, y: f64) {
        self.stage.lock().unwrap().x_mut().set_size(x);
        self.stage.lock().unwrap().y_mut().set_size(y);
    }

    pub fn set_bp_per_screen(&mut self, z: f64) {
        self.data_api.set_scale(z);
        self.stage.lock().unwrap().x_mut().set_bp_per_screen(z);
    }

    pub fn test_and_reset_redraw(&self) -> bool {
        self.stage.lock().unwrap().redraw_needed().test_and_reset()
    }
}
