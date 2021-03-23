use std::fmt;
use anyhow::{ self, anyhow as err };
use wasm_bindgen::JsValue;
use web_sys::console;
use serde_json::Value as JsonValue;
use crate::util::message::Message;

fn error_to_string(v: JsValue) -> String {
    let x : JsonValue = v.into_serde().unwrap();
    format!("{} {}",x.to_string(),v.as_string().unwrap_or("mystery error".to_string()))
}

pub(crate) fn console_error(s: &str) {
    console::log_1(&s.into());
}
