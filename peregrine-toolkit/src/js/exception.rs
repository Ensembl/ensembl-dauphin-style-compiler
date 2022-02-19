use wasm_bindgen::JsValue;

use crate::error_important;

pub fn js_result_to_option_console<T>(value: Result<T,JsValue>) -> Option<T> {
    match value {
        Ok(v) => Some(v),
        Err(e) => {
            error_important!("js error: {:?}",e);
            None
        }
    }
}
