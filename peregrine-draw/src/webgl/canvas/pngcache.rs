use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::lock;
use web_sys::{HtmlImageElement};

#[derive(Clone)]
pub struct PngCache {
    cache: Arc<Mutex<HashMap<String,HtmlImageElement>>>
}

impl PngCache {
    pub(super) fn new() -> PngCache {
        PngCache {
            cache: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub(super) fn get(&self, name: &str) -> Option<HtmlImageElement> {
        lock!(self.cache).get(name).cloned()
    }

    pub(super) fn set(&self, name: &str, element: HtmlImageElement) {
        lock!(self.cache).insert(name.to_string(),element);
    }
}