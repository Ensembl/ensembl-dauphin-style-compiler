use wasm_bindgen::prelude::*;

mod integration {
    mod bell;
    pub(crate) mod pgcommander;
    pub(crate) mod pgdauphin;
    pub(crate) mod pgblackbox;
    pub(crate) mod pgchannel;
    pub(crate) mod pgconsole;
    pub(crate) mod pgintegration;
    mod stream;
}

mod util {
    pub(crate) mod ajax;
    pub(crate) mod error;
    pub(crate) mod safeelement;
}

use std::collections::HashSet;
use anyhow::{ self, Context };
use blackbox::{ blackbox_enable };
use commander::{ cdr_timer };
use crate::integration::pgchannel::PgChannel;
use crate::integration::pgconsole::{ PgConsoleWeb, PgConsoleLevel };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgintegration::PgIntegration;
use crate::integration::pgblackbox::{ pgblackbox_setup, pgblackbox_sync, pgblackbox_endpoint };
use crate::util::error::{ js_throw, js_option };
use peregrine_core::{ 
    PgCommander, PgDauphin, ProgramLoader, Commander, RequestManager, Channel, ChannelLocation, StickStore, StickId, StickAuthorityStore,
    CountingPromise, PanelProgramStore, Scale, PanelRunStore, Panel, Focus, Track, PanelStore, DataStore, PeregrineObjects, PgCommanderTaskSpec,
    PeregrineApi
};
use peregrine_dauphin_queue::{ PgDauphinQueue };
use peregrine_dauphin::peregrine_dauphin;
pub use url::Url;

fn setup_commander() -> anyhow::Result<PgCommanderWeb> {
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let html = js_option(document.body().clone(),"cannot get body")?;
    let commander = PgCommanderWeb::new(&html)?;
    commander.start();
    Ok(commander)
}

struct PeregrineWeb {
    commander: PgCommanderWeb,
    api: PeregrineApi
}

impl PeregrineWeb {
    fn new() -> anyhow::Result<PeregrineWeb> {
        pgblackbox_setup();
        let commander = setup_commander().context("setting up commander")?;
        let integration = PgIntegration::new(PgChannel::new(PgConsoleWeb::new(30,30.)));
        let objects = PeregrineObjects::new(Box::new(integration),commander.clone())?;
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&objects);
        let api = PeregrineApi::new(objects.clone())?;
        api.ready();
        let mut out = PeregrineWeb {
            api, commander
        };
        out.setup()?;
        Ok(out)
    }

    #[cfg(blackbox)]
    fn setup_blackbox(&self) {
        pgblackbox_endpoint(Some(&Url::parse("http://localhost:4040/blackbox/data").expect("bad blackbox url")));
        blackbox_enable("notice");
        blackbox_enable("warn");
        blackbox_enable("error");
        self.core.add_task("blackbox-sender",5,None,None,Box::pin(pgblackbox_sync()));
    }

    #[cfg(not(blackbox))]
    fn setup_blackbox(&self) {
    }

    fn setup(&mut self) -> anyhow::Result<()> {
        self.setup_blackbox();
        Ok(())
    }
}

async fn old_test(objects: PeregrineObjects) -> anyhow::Result<()> {
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let el = document.get_element_by_id("loop").expect("missing element");
    let panel = Panel::new(StickId::new("homo_sapiens_GCA_000001405_27:1"),64,Scale::new(20),Focus::new(None),Track::new("gene"));
    let out = objects.panel_store.run(&panel).await?;
    el.set_inner_html(&format!("{:?}",out));
    Ok(())
}

async fn test(api: PeregrineApi) -> anyhow::Result<()> {
    api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    api.set_position(1000000.);
    api.set_scale(20.);
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
    let pg_web = js_throw(PeregrineWeb::new());
    pg_web.api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)));
    pg_web.commander.add_task("test",100,None,Some(10000.),Box::pin(test(pg_web.api.clone())));
    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    js_throw(test_fn());
    Ok(())
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}