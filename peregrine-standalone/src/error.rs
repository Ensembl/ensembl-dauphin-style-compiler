use std::fmt;
use anyhow::{ self, anyhow as err };
use wasm_bindgen::JsValue;
use web_sys::console;

pub fn js_option<T>(e: Option<T>, msg: &'static str) -> anyhow::Result<T> {
    e.ok_or_else(|| err!(msg))
}

pub(crate) fn console_error(s: &str) {
    unsafe {
        console::log_1(&s.into());
    }
}

pub(crate) fn js_warn(e: anyhow::Result<()>) {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{:?}",e));
        }
    }
}

pub fn js_throw<T>(e: anyhow::Result<T>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{:?}",e));
            panic!("panic");
        }
    }
}

pub(crate) fn display_error<T,E>(e: Result<T,E>) -> anyhow::Result<T> where E: fmt::Display {
    e.map_err(|e| err!(e.to_string()))
}
