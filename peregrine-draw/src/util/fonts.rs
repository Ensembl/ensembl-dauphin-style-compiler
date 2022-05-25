use std::{sync::{Arc, Mutex}, collections::HashSet};

use js_sys::Reflect;
use peregrine_toolkit::{lock};
use wasm_bindgen::JsValue;
use web_sys::FontFaceSet;

use crate::Message;

use super::promise::promise_to_future;

#[derive(Clone)]
pub(crate) struct Fonts {
    loaded: Arc<Mutex<Option<HashSet<String>>>>
}

impl Fonts {
    pub(crate) fn new() -> Result<Fonts,Message> {
        let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
        let document = window.document().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get document")))?;
        let available = Reflect::has(&document,&JsValue::from_str("fonts")) == Ok(true);
        let loaded = if available {
            Some(HashSet::new())
        } else {
            None
        };
        Ok(Fonts {
            loaded: Arc::new(Mutex::new(loaded))
        })
    }

    fn contains(&self, spec: &str) -> bool {
        lock!(self.loaded).as_ref().map(|x| x.contains(spec)).unwrap_or(true)
    }

    fn fonts_api(&self) -> Option<FontFaceSet> {
        let window = if let Some(x) = web_sys::window() { x } else { return None; };
        let document = if let Some(x) = window.document() { x } else { return None; };
        Some(document.fonts())
    }

    pub(crate) async fn load_font(&self, spec: &str) {
        if self.contains(spec) { return; }
        let fonts = if let Some(x) = self.fonts_api() { x } else { return; };
        let promise = fonts.load(spec);
        let result = promise_to_future(promise).await;
        if result.is_ok() {
            if let Some(fonts) = lock!(self.loaded).as_mut() {
                fonts.insert(spec.to_string());
            }
        }
    }
}
