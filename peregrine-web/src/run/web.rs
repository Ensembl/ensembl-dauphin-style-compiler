use anyhow::{ self, Context, anyhow as err };
use crate::integration::pgchannel::PgChannel;
use crate::integration::pgconsole::{ PgConsoleWeb };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use crate::shape::layers::programstore::ProgramStore;
use std::sync::{ Mutex, Arc };

#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use crate::util::error::{ js_option };
use peregrine_core::{ 
    Commander,
    PeregrineObjects,
    PeregrineApi,
    PeregrineConfig
};
use peregrine_dauphin::peregrine_dauphin;
use super::frame::run_animations;
pub use url::Url;
pub use web_sys::{ console, WebGlRenderingContext };
use crate::train::GlTrainSet;
use wasm_bindgen::JsCast;
use crate::shape::core::stage::Stage;
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
    pub api: PeregrineApi,
    pub trainset: GlTrainSet,
    pub webgl: Arc<Mutex<WebGlGlobal>>,
    stage: Stage
}

impl PeregrineWeb {
    pub fn new() -> anyhow::Result<PeregrineWeb> {
        let commander = setup_commander().context("setting up commander")?;
        let console = PgConsoleWeb::new(30,30.);
        let mut config = PeregrineConfig::new();
        config.set_f64("animate.fade.slow",500.);
        config.set_f64("animate.fade.fast",100.);
        let api = PeregrineApi::new()?;
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
        let stage = Stage::new();
        let trainset = GlTrainSet::new(&config,api.clone(),&stage)?;
        let web_data = PgIntegration::new(PgChannel::new(console.clone()),trainset.clone(),webgl.clone());
        let objects = PeregrineObjects::new(Box::new(web_data),commander.clone())?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&objects);
        api.ready(objects.clone());
        let mut out = PeregrineWeb {
            api, commander, trainset, stage,  webgl
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

    pub(crate) fn stage(&self) -> &Stage { &self.stage }

    // TODO redraw on change
    pub fn set_x_position(&mut self, x: f64) {
        self.api.set_position(x);
        self.stage.set_x_position(x);
    }

    pub fn set_y_position(&mut self, x: f64) {
        self.stage.set_y_position(x);
    }

    pub fn set_size(&mut self, x: f64, y: f64) {
        self.stage.set_size(x,y);
    }

    pub fn set_zoom(&mut self, z: f64) {
        self.api.set_scale(z);
        self.stage.set_zoom(z);
    }    
}
