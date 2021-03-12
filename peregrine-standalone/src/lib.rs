use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{ cdr_timer };
use peregrine_draw::{ PeregrineDraw, PeregrineDrawApi };
#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use peregrine_data::{ 
    StickId, Channel, ChannelLocation, Commander, Track
};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use peregrine_draw::{ js_option, js_throw };

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

async fn test(mut draw_api: PeregrineDraw) -> anyhow::Result<()> {
    draw_api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)))?;
    draw_api.add_track(Track::new("gene-pc-fwd"));
    //
    draw_api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut scale = 20.;
    for _ in 0..20 {
        pos += 500000.;
        scale *= 0.1;
        draw_api.set_x(pos);
        draw_api.set_bp_per_screen(scale);
        cdr_timer(1000.).await;
    }
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
//    let console = PgConsoleWeb::new(30,30.);
    let mut config = PeregrineConfig::new();
    config.set_f64("animate.fade.slow",500.);
    config.set_f64("animate.fade.fast",100.);
    let window = js_option(web_sys::window(),"cannot get window")?;
    let document = js_option(window.document(),"cannot get document")?;
    let canvas = js_option(document.get_element_by_id("trainset"),"canvas gone AWOL")?;
    let pg_web = js_throw(PeregrineDraw::new(config,canvas,|message| {}));
    let commander = pg_web.commander();
    commander.add_task("test",100,None,None,Box::pin(test(pg_web)));
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