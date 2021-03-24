mod error;
mod pgblackbox;

use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{ cdr_timer };
use peregrine_draw::{ PeregrineDraw, PeregrineDrawApi, Message, PeregrineDom };
#[cfg(blackbox)]
use crate::pgblackbox::{ pgblackbox_setup };
#[cfg(blackbox)]
use web_sys::console;

use peregrine_data::{Channel, ChannelLocation, Commander, DataMessage, StickId, Track};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use peregrine_draw::{ PgCommanderWeb };
use crate::error::{ js_option, js_throw };
use web_sys::{ Document, Element, HtmlCollection };

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

const HTML : &str = r#"
    <div class="container">
        <div class="sticky"></div>
        <div class="browser"><canvas class="browser-canvas"></canvas></div>
    </div>
"#;

const CSS : &str = r#"
    .browser {
        height: 1234px;
    }

    .sticky {
        background: url(http://www.ensembl.info/wp-content/uploads/2018/02/weird_genomes_4tile2.png) repeat;
        position: sticky;
        top: 0;
        height: 100%;
    }
"#;

fn unique_element(c: HtmlCollection) -> Result<Element,Message> {
    if c.length() != 1 { return Err(Message::BadTemplate(format!("collection has {} members, expected singleton",c.length()))) }
    c.item(0).ok_or_else(|| Message::BadTemplate(format!("collection has {} members, expected singleton",c.length())))
}

fn add_css(document: &Document, css: &str) -> Result<(),Message> {
    let style = document.create_element("style").map_err(|e| Message::ConfusedWebBrowser(format!("Cannot create style element")))?;
    style.set_text_content(Some(css));
    style.set_attribute("type","text/css").map_err(|e| Message::ConfusedWebBrowser(format!("Cannot set style element attr")))?;
    let body = document.body().ok_or_else(|| Message::ConfusedWebBrowser(format!("Document has no body")))?;
    body.append_with_node_1(&style);
    Ok(())
}

// TODO id prefixes
fn create_framework_at(el: &Element) -> Result<PeregrineDom,Message> {
    el.set_inner_html(HTML);
    add_css(&el.owner_document().ok_or_else(|| Message::ConfusedWebBrowser(format!("Element has no document")))?,CSS);
    let canvas = unique_element(el.get_elements_by_class_name("browser-canvas"))?;
    PeregrineDom::new(canvas)
}

fn test_fn() -> Result<(),Message> {
    let mut config = PeregrineConfig::new();
    config.set_f64("animate.fade.slow",500.);
    config.set_f64("animate.fade.fast",100.);
    let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
    let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get document")))?;
    let browser_el = document.get_element_by_id("browser").ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get canvas")))?;
    let dom = create_framework_at(&browser_el)?;
    let pg_web = js_throw(PeregrineDraw::new(config,dom,|message| {
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