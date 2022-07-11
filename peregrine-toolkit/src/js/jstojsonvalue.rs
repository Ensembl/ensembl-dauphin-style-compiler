use serde_json::{ Value as JsonValue, Number as JsonNumber, Map as JsonMap };
use wasm_bindgen::{JsValue, JsCast};
use js_sys::{ Array, Object };

fn js_typeof(js: &JsValue) -> Result<String,()> {
    Ok(js.js_typeof().as_string().ok_or(())?)
}

fn js_object_to_json(js: &JsValue) -> Result<JsonValue,()> {
    let input : Object = js.clone().dyn_into().unwrap();
    let mut out = JsonMap::new();
    for entry in Object::entries(&input).iter() {
        let entry = Array::from(&entry);
        let key = entry.get(0).as_string().ok_or(())?;
        let value = js_to_json(&entry.get(1))?;
        out.insert(key,value);
    }
    Ok(JsonValue::Object(out))
}

fn js_array_to_json(js: &JsValue) -> Result<JsonValue,()> {
    let array = Array::from(js);
    Ok(JsonValue::Array(array.iter().map(|x| js_to_json(&x)).collect::<Result<Vec<_>,_>>()?))
}

pub fn js_to_json(js: &JsValue) -> Result<JsonValue,()> {
    if js.is_object() && !js.is_function() && !Array::is_array(js) {
        js_object_to_json(js)
    } else {
        match js_typeof(js)?.as_str() {
            "object" => {
                if Array::is_array(js) {
                    js_array_to_json(js)
                } else {
                    return Err(());
                }
            },
            "number" => Ok(JsonValue::Number(JsonNumber::from_f64(js.as_f64().unwrap()).unwrap())),
            "boolean" => Ok(JsonValue::Bool(js.as_bool().unwrap())),
            "string" => Ok(JsonValue::String(js.as_string().unwrap())),
            _ => { return Err(()) }
        }
    }
}
