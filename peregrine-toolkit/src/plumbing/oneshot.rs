use std::sync::{Arc, Mutex};

use crate::lock;

#[derive(Clone)]
pub struct OneShot {
    callbacks: Arc<Mutex<Option<Vec<Box<dyn FnOnce()>>>>>
}

impl OneShot {
    pub fn new() -> OneShot {
        OneShot {
            callbacks: Arc::new(Mutex::new(Some(vec![])))
        }
    }

    pub fn add<F>(&self, cb: F) where F: FnOnce() + 'static {
        if let Some(callbacks) = &mut *lock!(self.callbacks) {
            callbacks.push(Box::new(cb));
        } else {
            cb();
        }
    }

    pub fn run(&self) {
        if let Some(callbacks) = lock!(self.callbacks).take() {
            for callback in callbacks {
                callback();
            }
        }
    }

    pub fn poll(&self) -> bool { lock!(self.callbacks).is_none() }
}
