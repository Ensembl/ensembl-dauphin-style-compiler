use std::collections::HashMap;

use js_sys::{Array, Reflect};
use peregrine_data::{ReceivedData, DataAlgorithm};
use peregrine_toolkit::error::Error;
use wasm_bindgen::JsValue;

use crate::{jsutil::{to_hashmap, to_string, emap}, backend::CallbackError};

fn array_to_code(value: &Array) -> Result<String,CallbackError> {
    let out = if value.length() == 0 {
        "E"
    } else {
        let first = value.get(0);
        if first.as_f64().is_some() {
            "NRA"
        } else if first.as_bool().is_some() {
            "BA"
        } else if first.is_string() {
            "SA"
        } else {
            return emap(Err(Error::operr("unknown array")));
        }
    };
    Ok(out.to_string())
}

/* Arrays of numbers, strings, bools, get NRA, SA, BA.
 * Objects use "code" and "data" keys to specify directly.
 */
fn add_alg_context(data: JsValue) -> Result<JsValue,CallbackError> {
    let out = Array::new();
    let (code,data) = if Array::is_array(&data) {
        let input = Array::from(&data);
        let code = array_to_code(&input)?;
        (code,input)
    } else {
        let code = to_string(&emap(Reflect::get(&data,&JsValue::from("code")).map_err(|_| Error::operr("missing code key")))?)?;
        let data = emap(Reflect::get(&data,&JsValue::from("data")).map_err(|_| Error::operr("missing data key")))?.into();
        (code,data)
    };
    out.set(0,code.into());
    out.set(1,data.into());
    Ok(out.into())
}

fn ds_one_datastream(v: JsValue) -> Result<ReceivedData,CallbackError> {
    let x : Result<DataAlgorithm,_> = serde_wasm_bindgen::from_value(add_alg_context(v)?);
    let y = emap(x.map_err(|e| Error::operr(&format!("cannot deserialize: {:?}",e))))?;
    emap(y.to_received_data().map_err(|_| Error::operr("cannot deserialize")))
}

pub(super) fn ds_all_datastreams(data: JsValue) -> Result<HashMap<String,ReceivedData>,CallbackError> {
    to_hashmap(data)?.drain().map(|(k,v)| {
        Ok((k,ds_one_datastream(v)?))
    }).collect()
}
