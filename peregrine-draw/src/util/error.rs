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

pub(crate) fn js_error<T>(e: Result<T,JsValue>) -> Result<T,Message> {
    e.map_err(|e| Message::XXXTmp(error_to_string(e)))
}

pub(crate) fn display_error<T,E>(e: Result<T,E>) -> Result<T,Message> where E: fmt::Display {
    e.map_err(|e| Message::XXXTmp(e.to_string()))
}

pub fn js_option<T>(e: Option<T>, msg: &'static str) -> Result<T,Message> {
    e.ok_or_else(|| Message::XXXTmp(msg.to_string()))
}

pub(crate) fn console_error(s: &str) {
    unsafe {
        console::log_1(&s.into());
    }
}

pub(crate) fn js_warn(e: Result<(),Message>) {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{}",e));
        }
    }
}

pub fn js_throw<T>(e: Result<T,Message>) -> T {
    match e {
        Ok(e) => e,
        Err(e) => {
            console_error(&format!("{}",e));
            panic!("panic");
        }
    }
}
