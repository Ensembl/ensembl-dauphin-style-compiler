use std::fmt;
use anyhow::{ self, anyhow as err };
use wasm_bindgen::JsValue;
use web_sys::console;
use peregrine_draw::Message;

pub(crate) fn console_error(s: &str) {
    console::log_1(&s.into());
}

pub fn js_throw<T>(e: Result<T,Message>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{:?}",e));
            panic!("panic");
        }
    }
}
