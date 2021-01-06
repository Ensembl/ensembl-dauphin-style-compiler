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

mod run {
    pub(crate) mod web;
    mod frame;
}

mod shape {
    pub(crate) mod glshape;
}

mod train {
    mod glcarriage;
    mod gltrain;
    mod gltrainset;

    pub(crate) use self::gltrainset::GlTrainSet;
}

mod util {
    pub(crate) mod ajax;
    pub(crate) mod error;
    pub(crate) mod safeelement;
}

use anyhow::{ self };
use commander::{ cdr_timer };
use crate::run::web::PeregrineWeb;
#[cfg(blackbox)]
use crate::integration::pgblackbox::{ pgblackbox_setup };
use crate::util::error::{ js_throw, js_option };
use peregrine_core::{ 
    StickId, PeregrineApi, Channel, ChannelLocation, Commander
};
pub use url::Url;

#[cfg(blackbox)]
use blackbox::{ blackbox_enable, blackbox_log };

async fn test(api: PeregrineApi) -> anyhow::Result<()> {
    api.set_stick(&StickId::new("homo_sapiens_GCA_000001405_27:1"));
    let mut pos = 2500000.;
    let mut scale = 20.;
    for _ in 0..20 {
        pos += 500000.;
        scale += 0.1;
        api.set_position(pos);
        api.set_scale(scale);
        cdr_timer(1000.).await;
    }
    Ok(())
}

fn test_fn() -> anyhow::Result<()> {
    let pg_web = js_throw(PeregrineWeb::new());
    pg_web.api.bootstrap(Channel::new(&ChannelLocation::HttpChannel(Url::parse("http://localhost:3333/api/data")?)));
    pg_web.commander.add_task("test",100,None,None,Box::pin(test(pg_web.api.clone())));
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