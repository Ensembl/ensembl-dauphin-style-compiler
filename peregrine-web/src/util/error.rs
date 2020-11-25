use std::fmt;
use anyhow::{ self, anyhow as err };
use wasm_bindgen::JsValue;
use web_sys::console;
use serde_json::Value as JsonValue;

fn error_to_string(v: JsValue) -> String {
    let x : JsonValue = v.into_serde().unwrap();
    format!("{} {}",x.to_string(),v.as_string().unwrap_or("mystery error".to_string()))
}

pub(crate) fn js_error<T>(e: Result<T,JsValue>) -> anyhow::Result<T> {
    e.map_err(|e| err!(error_to_string(e)))
}

pub(crate) fn display_error<T,E>(e: Result<T,E>) -> anyhow::Result<T> where E: fmt::Display {
    e.map_err(|e| err!(e.to_string()))
}

pub(crate) fn js_option<T>(e: Option<T>, msg: &'static str) -> anyhow::Result<T> {
    e.ok_or_else(|| err!(msg))
}

pub(crate) fn console_error(s: &str) {
    //console::log_1(&s.into());
}

pub(crate) fn js_warn(e: anyhow::Result<()>) {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{:?}",e));
        }
    }
}

pub(crate) fn js_throw<T>(e: anyhow::Result<T>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{:?}",e));
            panic!("panic");
        }
    }
}
