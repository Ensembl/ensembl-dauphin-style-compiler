mod error;
mod pgblackbox;

use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{ cdr_timer };
use peregrine_draw::{ PeregrineDraw, PeregrineDrawApi, Message };
#[cfg(blackbox)]
use crate::pgblackbox::{ pgblackbox_setup };
#[cfg(blackbox)]
use web_sys::console;

use peregrine_data::{Channel, ChannelLocation, Commander, DataMessage, StickId, Track};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use peregrine_draw::{ js_option, js_throw, PgCommanderWeb };

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

#[cfg(blackbox)]
fn setup_blackbox(commander: &PgCommanderWeb) {
    let mut ign = pgblackbox_setup();
    ign.set_url(&Url::parse("http://localhost:4040/blackbox/data").expect("bad blackbox url"));
    let ign2 = ign.clone();
    blackbox_enable("notice");
    blackbox_enable("warn");
    blackbox_enable("error");
    commander.add::<()>("blackbox",10,None,None,Box::pin(async move { ign2.sync_task().await; Ok(()) }));
    blackbox_log("general","blackbox configured");
    console::log_1(&format!("blackbox configured").into());
}

#[cfg(not(blackbox))]
fn setup_blackbox(c_ommander: &PgCommanderWeb) {
}

async fn test(mut draw_api: PeregrineDraw) -> anyhow::Result<()> {
    draw_api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)));
    draw_api.add_track(Track::new("gene-pc-fwd"));
    draw_api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut bp_per_screen = 1000000.;
    for _ in 0..20 {
        pos += 50000.;
        draw_api.set_x(pos);
        draw_api.set_bp_per_screen(bp_per_screen);
        bp_per_screen *= 0.95;
        cdr_timer(1000.).await;
    }
    Ok(())
}

fn test_fn() -> Result<(),Message> {
    let mut config = PeregrineConfig::new();
    config.set_f64("animate.fade.slow",500.);
    config.set_f64("animate.fade.fast",100.);
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let canvas = js_option(document.get_element_by_id("trainset"),"canvas gone AWOL")?;
    let pg_web = js_throw(PeregrineDraw::new(config,canvas,|message| {
        use web_sys::console;
        console::log_1(&format!("{}",message).into());
    }));
    let commander = pg_web.commander();
    commander.add("test",100,None,None,Box::pin(test(pg_web)));
    setup_blackbox(&commander);
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