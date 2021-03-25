use std::fmt::Debug;
mod standalonedom;
use wasm_bindgen::prelude::*;
use anyhow::{ self };
use commander::{ cdr_timer };
use peregrine_draw::{ PeregrineDraw, PeregrineDrawApi, Message };
use peregrine_data::{Channel, ChannelLocation, StickId, Track};
use peregrine_data::{ PeregrineConfig };
pub use url::Url;
use crate::standalonedom::make_dom;
use web_sys::console;

/*
 * This utility just catches handles serious errors in setting up this demonstration. It's not the main error-handling
 * code.
 */
pub fn js_throw<T,E: Debug>(e: Result<T,E>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console::error_1(&format!("{:?}",e).into());
            panic!("deliberate panic from js_throw following error. Ignore this trace, see error above.");
        }
    }
}

/*
 * This function just does some daft stuff to kick the tyres for now. Don't worry too much about it. But it is quite
 * a good example of the sort of API calls you might make on receiving events from the browser chrome. The exact
 * name and arguments to this API are still up in the air, but you get the idea....
 */
async fn test(mut draw_api: PeregrineDraw) -> anyhow::Result<()> {
    draw_api.set_size(100.,100.);
    draw_api.add_track(Track::new("gene-pc-fwd"));
    draw_api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut bp_per_screen = 1000000.;
    for _ in 0..20 {
        pos += 50000.;
        draw_api.set_x(pos);
        draw_api.set_bp_per_screen(bp_per_screen);
        bp_per_screen *= 0.95;
        cdr_timer(1000.).await; // Wait one second
    }
    Ok(())
}

fn setup_genome_browser() -> Result<(),Message> {
    /*
     * Set some config options. There aren't many of these yet but there will probably be more. The idea is that
     * things like the speed of changes and positions, colours, cache sizes, etc, can come from the surrounding app.
     * This isn't about the data in the tracks itself but there's probably going to be a whole load of configurable
     * oojimaflips associated with the browser in the end.
     */
    let mut config = PeregrineConfig::new();
    config.set_f64("animate.fade.slow",500.);
    config.set_f64("animate.fade.fast",100.);
    /*
     * Here we call standalonedom.rs which sorts out finding an element and setting it up for the genome browser to
     * use. See that file for details.
     */
    let dom = make_dom()?;
    /*
     * Create a genome browser object.
     */
    let mut draw_api = js_throw(PeregrineDraw::new(config,dom));
    /* 
     * Configure the message reporter. This gets notifications of errors, etc. Note that this may be async to the
     * request which caused it (eg it may be after a fair few AJAX calls that we finally realise some data we supplied
     * was dodgy). Shortly there should be a mechanism to tie the message and originating request together.
     */
    draw_api.set_message_reporter(|message| {
        console::error_1(&format!("{}",message).into());
    });
    /*
     * In general integrations probably don't want to set up blackbox, but I do here. It's a useful debug and
     * performance-tweaking tool. Just don't call this if you don't care.
     */
    js_throw(draw_api.setup_blackbox("http://localhost:4040/blackbox/data"));
    /*
     * Ok, we're ready to go. Bootstrapping causes the genome browser to go to the backend and configure itself.
     */
    let url = "http://localhost:3333/api/data";
    draw_api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(js_throw(Url::parse(url)))));
    /*
     * For now just start an async process to do some daft stuff to kick the tyres.
     */
    let commander = draw_api.commander();
    commander.add("test",100,None,None,Box::pin(test(draw_api)));
    Ok(())
}

/*
 * This is the code which starts it all.
 */
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    js_throw(setup_genome_browser());
    Ok(())
}

/*
 * This is an obscure thing which makes stack traces better.
 */
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
