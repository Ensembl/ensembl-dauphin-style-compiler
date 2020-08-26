use wasm_bindgen::prelude::*;

mod integration {
    mod bell;
    pub(crate) mod pgcommander;
    pub(crate) mod pgdauphin;
    pub(crate) mod pgblackbox;
    pub(crate) mod pgchannel;
    pub(crate) mod pgconsole;
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
use crate::integration::pgblackbox::{ pgblackbox_setup, pgblackbox_sync, pgblackbox_endpoint };
use crate::util::error::{ js_throw, js_option };
use peregrine_core::{ PgCore, PgCommander, PgDauphin, ProgramLoader, Commander, RequestManager, Channel, ChannelLocation, StickStore, StickId, StickAuthorityStore };
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

//static PEREGRINEWEB: Lazy<Mutex<PeregrineWeb>> = Lazy::new(|| Mutex::new(js_throw(PeregrineWeb::new())));

struct PeregrineWeb {
    core: PgCore,
    stick_store: StickStore
}

impl PeregrineWeb {
    fn new() -> anyhow::Result<PeregrineWeb> {
        pgblackbox_setup();
        let console = PgConsoleWeb::new(60,60.);
        let commander = PgCommander::new(Box::new(setup_commander().context("setting up commander")?)); 
        let mut manager = RequestManager::new(PgChannel::new(Box::new(console.clone())),&commander);
        let stick_store = StickStore::new(&commander,&manager,&Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)))?;
        let pdq = PgDauphinQueue::new();
        let dauphin = PgDauphin::new(&pdq)?;
        let loader = ProgramLoader::new(&commander,&manager,&dauphin)?;
        let stick_authority_store = StickAuthorityStore::new(&commander,&manager,&loader,&dauphin);
        peregrine_dauphin(Box::new(PgDauphinIntegrationWeb()),&commander,&pdq,&manager,&stick_authority_store);
        manager.add_receiver(Box::new(dauphin.clone()));
        let mut out = PeregrineWeb {
            core: PgCore::new(&commander,&dauphin,&manager,&stick_authority_store)?,
            stick_store: stick_store.clone()
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

async fn test(core: PgCore, stick_store: StickStore) -> anyhow::Result<()> {
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let el = document.get_element_by_id("loop").expect("missing element");
    cdr_timer(5000.).await;
    let tags = core.stick_authority_store.lookup(StickId::new("homo_sapiens_GCA_000001405_27:1")).await?.map(|x| x.tags().clone()).unwrap_or(HashSet::new());
    cdr_timer(1000.).await;
    el.set_inner_html(&format!("{:?}",tags));
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
    let mut pg_web = js_throw(PeregrineWeb::new());
    pg_web.core.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)))?;
    pg_web.core.add_task("test",100,None,Some(10000.),Box::pin(test(pg_web.core.clone(),pg_web.stick_store.clone())));
    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    js_throw(test_fn());
    Ok(())
}
