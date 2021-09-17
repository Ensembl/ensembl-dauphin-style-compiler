use web_sys::console;
use wasm_bindgen::JsValue;

pub fn js_result_to_option_console<T>(value: Result<T,JsValue>) -> Option<T> {
    match value {
        Ok(v) => Some(v),
        Err(e) => {
            console::error_1(&format!("js error: {:?}",e).into());
            None
        }
    }
}
