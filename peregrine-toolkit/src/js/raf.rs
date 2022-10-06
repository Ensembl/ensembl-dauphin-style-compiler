use std::sync::{Arc, Mutex};

use crate::lock;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Window;

pub struct Raf {
    window: Window,
    handle: Arc<Mutex<Option<i32>>>,
    #[allow(unused)]
    js_closure: Closure<dyn FnMut()>

}

impl Raf {
    pub fn new<F>(mut cb: F) -> Raf where F: FnMut() + 'static {
        let handle = Arc::new(Mutex::new(None));
        let window = web_sys::window().unwrap(); // XXX errors
        let handle2 = handle.clone();
        let js_closure = Closure::wrap(Box::new(move || { 
            *lock!(handle2) = None;
            cb()
        }) as Box<dyn FnMut()>);
        Raf { window, handle, js_closure }
    }

    pub fn go(&mut self) {
        let mut handle = lock!(self.handle);
        if handle.is_none() {
            *handle = Some(self.window.request_animation_frame(self.js_closure.as_ref().unchecked_ref()).ok().unwrap());
        }
        drop(handle);
    }

    pub fn stop(&mut self) {
        let mut handle = lock!(self.handle);
        if let Some(handle) = handle.as_mut() {
            self.window.cancel_animation_frame(*handle);
        }
        *handle = None;
    }
}

impl Drop for Raf {
    fn drop(&mut self) {
        self.stop();
    }
}