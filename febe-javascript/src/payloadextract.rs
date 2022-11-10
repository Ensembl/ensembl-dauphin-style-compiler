use js_sys::{Reflect, Object, JsString};
use peregrine_data::BackendNamespace;
use peregrine_toolkit::{error::Error };
use serde_wasm_bindgen::to_value;
use wasm_bindgen::JsValue;
use crate::callbacks::Callbacks;
use crate::jsutil::to_hashmap;

pub struct JsBackendThis {
    payload: JsValue,
    backend_namespace: BackendNamespace
}

impl JsBackendThis {
    fn do_to_jsvalue(&self) -> Result<JsValue,serde_wasm_bindgen::Error> {
        let out = Object::new();
        let js_backend_namespace = to_value(&self.backend_namespace)?;
        Reflect::set(&out, &JsString::from("backend_namespace"), &js_backend_namespace)?;
        Reflect::set(&out, &JsString::from("payload"), &self.payload)?;
        Ok(out.into())
    }


    fn to_jsvalue(&self) -> Result<JsValue,Error> {
        self.do_to_jsvalue().map_err(|e| Error::operr(&format!("building this: {}",e)))
    }
}

pub(crate) struct PayloadExtractor {
    pub(crate) callbacks: Callbacks
}

impl PayloadExtractor {
    pub(crate) fn new(payload: JsValue, backend_namespace: &BackendNamespace) -> Result<PayloadExtractor,Error> {
        let mut top_level = to_hashmap(payload)?;
        let this = JsBackendThis {
            payload: top_level.remove("payload").unwrap_or(JsValue::NULL),
            backend_namespace: backend_namespace.clone()
        };
        let mut callbacks = Callbacks::new(this.to_jsvalue()?);
        if let Some(callbacks_js) = top_level.remove("callbacks") {
            for (name,value) in to_hashmap(callbacks_js)? {
                callbacks.add(&name,value)?;
            }
        }
        Ok(PayloadExtractor {
            callbacks
        })
    }
}
