use peregrine_toolkit::{error::Error };
use wasm_bindgen::JsValue;
use crate::callbacks::Callbacks;
use crate::jsutil::to_hashmap;

pub(crate) struct PayloadExtractor {
    pub(crate) callbacks: Callbacks
}

impl PayloadExtractor {
    pub(crate) fn new(payload: JsValue) -> Result<PayloadExtractor,Error> {
        let mut top_level = to_hashmap(payload)?;
        let this = top_level.remove("this");
        let mut callbacks = Callbacks::new(this);
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
