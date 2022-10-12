use std::collections::HashMap;
use js_sys::Function;
use js_sys::JsString;
use js_sys::Number;
use peregrine_toolkit::{error::Error };
use wasm_bindgen::{ JsValue, JsCast };
use js_sys::Object as JsObject;
use js_sys::Array as JsArray;

pub(crate) fn to_array(value: JsValue) -> Result<JsArray,Error> {
    value.dyn_into().map_err(|e| Error::operr(&format!("expected array: {:?}",e)))
}

pub(crate) fn to_object(value: JsValue) -> Result<JsObject,Error> {
    value.dyn_into().map_err(|e| Error::operr(&format!("expected map: {:?}",e)))
}

pub(crate) fn to_string(value: JsValue) -> Result<String,Error> {
    let s : JsString = value.dyn_into().map_err(|e| Error::operr(&format!("expected string: {:?}",e)))?;
    Ok(s.into())
}

pub(crate) fn to_int(value: JsValue) -> Result<i64,Error> {
    let s : Number = value.dyn_into().map_err(|e| Error::operr(&format!("expected float: {:?}",e)))?;
    Ok(s.value_of().round() as i64)
}

pub(crate) fn to_function(value: JsValue) -> Result<Function,Error> {
    value.dyn_into().map_err(|e| Error::operr(&format!("expected array: {:?}",e)))
}

pub(crate) fn to_hashmap(value: JsValue) -> Result<HashMap<String,JsValue>,Error> {
    let mut out = HashMap::new();
    let iterator = JsObject::entries(&to_object(value)?);
    for entry in iterator.iter() {
        let kv = to_array(entry)?;
        let key = to_string(kv.get(0))?;
        out.insert(key.to_string(),kv.get(1));
    }
    Ok(out)
}
