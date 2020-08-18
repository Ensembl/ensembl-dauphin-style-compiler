use wasm_bindgen::prelude::*;
use std::future::Future;
use commander::RunSlot;
use web_sys::console;
use once_cell::sync::Lazy;

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

use std::sync::{ Arc, Mutex };
use anyhow::{ self, Context };
use blackbox::{ blackbox_enable };
use commander::{ cdr_tick, cdr_timer };
use crate::integration::pgchannel::PgChannel;
use crate::integration::pgconsole::PgConsole;
use crate::integration::pgcommander::PgCommanderWeb;
use crate::integration::pgdauphin::PgDauphinIntegrationWeb;
use crate::integration::pgblackbox::{ pgblackbox_setup, pgblackbox_sync, pgblackbox_endpoint };
use crate::util::error::{ js_throw, js_option };
use serde_cbor::Value as CborValue;
use peregrine_core::{ PgCore, PgCommander, PgDauphin, Commander, RequestManager };

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
    core: PgCore
}

impl PeregrineWeb {
    fn new() -> anyhow::Result<PeregrineWeb> {
        pgblackbox_setup();
        let commander = PgCommander::new(Box::new(setup_commander().context("setting up commander")?)); 
        let dauphin = PgDauphin::new(Box::new(PgDauphinIntegrationWeb()))?;
        let console = PgConsole::new(10,30.);
        let manager = RequestManager::new(PgChannel::new(&console),&dauphin,&commander);
        let mut out = PeregrineWeb {
            core: PgCore::new(&commander,&dauphin,&manager)?
        };
        out.setup()?;
        Ok(out)
    }

    #[cfg(blackbox)]
    fn setup_blackbox(&self) {
        pgblackbox_endpoint(Some("http://localhost:4040/blackbox/data"));
        blackbox_enable("notice");
        blackbox_enable("warn");
        blackbox_enable("error");
        self.commander.add_task("blackbox-sender",5,None,None,pgblackbox_sync());
    }

    #[cfg(not(blackbox))]
    fn setup_blackbox(&self) {
    }

    fn setup(&mut self) -> anyhow::Result<()> {
        self.setup_blackbox();
        Ok(())
    }
}

async fn test(frames: Arc<Mutex<u32>>) -> anyhow::Result<()> {
    loop {
        cdr_timer(1000.).await;
        let window = js_option(web_sys::window(),"cannot get window")?;
        let document = js_option(window.document(),"cannot get document")?;
        let el = document.get_element_by_id("loop").expect("missing element");
        el.set_inner_html(&format!("{}",frames.lock().unwrap()));
    }
}

async fn test2(frames: Arc<Mutex<u32>>) -> anyhow::Result<()> {
    loop {
        cdr_tick(1).await;
        *frames.lock().unwrap() += 1;
    }
}

fn test_fn() -> anyhow::Result<()> {
    let mut pg_web = js_throw(PeregrineWeb::new());
    let frames = Arc::new(Mutex::new(0_u32));
    pg_web.core.add_task("test",100,None,Some(10000.),Box::pin(test(frames.clone())));
    pg_web.core.add_task("test2",100,None,Some(5000.),Box::pin(test2(frames)));
    let test = include_bytes!("test.dpb");
    let prog : CborValue = serde_cbor::from_slice(test)?;
    pg_web.core.add_binary(&prog)?;
    pg_web.core.run("hello",0,None,None).expect("run");
    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    js_throw(test_fn());
    Ok(())
}
