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
    mod stream;
}

mod util {
    pub(crate) mod error;
    pub(crate) mod safeelement;
}

use std::sync::{ Arc, Mutex };
use anyhow::{ self, Context };
use blackbox::{ blackbox_enable };
use commander::{ cdr_tick, cdr_timer };
use crate::integration::pgcommander::PgCommander;
use crate::integration::pgdauphin::PgDauphin;
use crate::integration::pgblackbox::{ pgblackbox_setup, pgblackbox_sync, pgblackbox_endpoint };
use crate::util::error::{ js_throw, js_option };
use serde_cbor::Value as CborValue;

fn setup_commander() -> anyhow::Result<PgCommander> {
    let window = js_option(web_sys::window())?;
    let document = js_option(window.document())?;
    let html = js_option(document.body().clone())?;
    let commander = PgCommander::new(&html)?;
    commander.start();
    Ok(commander)
}

//static PEREGRINEWEB: Lazy<Mutex<PeregrineWeb>> = Lazy::new(|| Mutex::new(js_throw(PeregrineWeb::new())));

struct PeregrineWeb {
    commander: PgCommander,
    dauphin: PgDauphin
}

impl PeregrineWeb {
    fn new() -> anyhow::Result<PeregrineWeb> {
        pgblackbox_setup();
        let mut out = PeregrineWeb {
            commander: setup_commander().context("setting up commander")?,
            dauphin: PgDauphin::new()?
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

    pub fn add_task<F>(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: F) where F: Future<Output=anyhow::Result<()>> + 'static {
        self.commander.add_task(name,prio,slot,timeout,f)
    }

    pub fn run(&mut self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>) -> anyhow::Result<()> {
        let (dauphin,commander) = (&mut self.dauphin, &mut self.commander);
        let task = dauphin.load(name)?;
        commander.add_task(&format!("dauphin: '{}'",name),prio,slot,timeout,task.run());
        Ok(())
    }

    pub fn add_binary(&mut self, cbor: &CborValue) -> anyhow::Result<()> {
        self.dauphin.add_binary(cbor)
    }
}

async fn test(frames: Arc<Mutex<u32>>) -> anyhow::Result<()> {
    loop {
        cdr_timer(1000.).await;
        let window = js_option(web_sys::window())?;
        let document = js_option(window.document())?;
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
    pg_web.add_task("test",100,None,Some(10000.),test(frames.clone()));
    pg_web.add_task("test2",100,None,Some(5000.),test2(frames));
    let test = include_bytes!("test.dpb");
    let prog : CborValue = serde_cbor::from_slice(test)?;
    pg_web.add_binary(&prog)?;
    pg_web.run("hello",0,None,None).expect("run");
    Ok(())
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    js_throw(test_fn());
    Ok(())
}
