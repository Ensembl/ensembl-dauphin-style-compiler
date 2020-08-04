use anyhow::{ self, anyhow as err };
use wasm_bindgen::JsValue;
use web_sys::console;

pub(crate) fn js_error<T>(e: Result<T,JsValue>) -> anyhow::Result<T> {
    e.map_err(|e| err!(e.as_string().unwrap_or("mystery error".to_string())))
}

pub(crate) fn js_option<T>(e: Option<T>) -> anyhow::Result<T> {
    e.ok_or_else(|| err!("unexpected unwrap failure"))
}

pub(crate) fn console_error(s: &str) {
    console::error_1(&s.into());
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
