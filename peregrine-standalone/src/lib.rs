mod error;
mod standalonedom;
use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{ cdr_timer };
use peregrine_draw::{ PeregrineDraw, PeregrineDrawApi, Message };
use peregrine_data::{Channel, ChannelLocation, StickId, Track};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use peregrine_draw::{ PgCommanderWeb };
use crate::error::{ js_throw };
use crate::standalonedom::make_dom;

async fn test(mut draw_api: PeregrineDraw) -> anyhow::Result<()> {
    draw_api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)));
    draw_api.setup_blackbox("http://localhost:4040/blackbox/data");
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
    let dom = make_dom()?;
    let pg_web = js_throw(PeregrineDraw::new(config,dom,|message| {
        use web_sys::console;
        console::log_1(&format!("{}",message).into());
    }));
    let commander = pg_web.commander();
    commander.add("test",100,None,None,Box::pin(test(pg_web)));
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
