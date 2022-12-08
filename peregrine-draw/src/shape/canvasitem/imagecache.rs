use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::lock;
use web_sys::{HtmlImageElement};

#[derive(Clone)]
pub struct ImageCache {
    cache: Arc<Mutex<HashMap<String,HtmlImageElement>>>
}

impl ImageCache {
    pub(crate) fn new() -> ImageCache {
        ImageCache {
            cache: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<HtmlImageElement> {
        lock!(self.cache).get(name).cloned()
    }

    pub(crate) fn set(&self, name: &str, element: HtmlImageElement) {
        lock!(self.cache).insert(name.to_string(),element);
    }
}
