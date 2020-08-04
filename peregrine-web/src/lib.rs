use wasm_bindgen::prelude::*;

mod scheduler {
    mod bell;
    pub(crate) mod pgcommander;
}

mod util {
    pub(crate) mod error;
    pub(crate) mod safeelement;
}

use std::sync::{ Arc, Mutex };
use anyhow::{ self, Context };
use commander::{ cdr_tick, cdr_timer };
use crate::scheduler::pgcommander::PgCommander;
use crate::util::error::{ js_throw, js_option };

fn setup_commander() -> anyhow::Result<PgCommander> {
    let window = js_option(web_sys::window())?;
    let document = js_option(window.document())?;
    let html = js_option(document.body().clone())?;
    let commander = PgCommander::new(&html)?;
    commander.start();
    Ok(commander)
}

fn setup() -> anyhow::Result<PgCommander> {
    let pgc = setup_commander().context("setting up commander")?;
    Ok(pgc)
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

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let pgc = js_throw(setup());
    let frames = Arc::new(Mutex::new(0_u32));
    pgc.add_task("test",100,None,Some(10000.),test(frames.clone()));
    pgc.add_task("test2",100,None,Some(5000.),test2(frames));
    Ok(())
}
