use anyhow::{ self, Context };
use crate::integration::pgchannel::PgChannel;
use crate::integration::pgconsole::{ PgConsoleWeb };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
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
pub use web_sys::console;
use crate::train::GlTrainSet;

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

#[derive(Clone)]
pub struct PeregrineWeb {
    pub commander: PgCommanderWeb,
    pub api: PeregrineApi,
    pub trainset: GlTrainSet
}

impl PeregrineWeb {
    pub fn new() -> anyhow::Result<PeregrineWeb> {
        let commander = setup_commander().context("setting up commander")?;
        let console = PgConsoleWeb::new(30,30.);
        let mut config = PeregrineConfig::new();
        config.set_f64("animate.fade.slow",500.);
        config.set_f64("animate.fade.fast",100.);
        let api = PeregrineApi::new()?;
        let trainset = GlTrainSet::new(&config,api.clone());
        let integration = PgIntegration::new(PgChannel::new(console.clone()),trainset.clone());
        let objects = PeregrineObjects::new(Box::new(integration),commander.clone())?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&objects);
        api.ready(objects.clone());
        let mut out = PeregrineWeb {
            api, commander, trainset
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
}
