use std::sync::{Arc, Mutex};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Window;
use crate::lock;

pub struct Timer {
    window: Window,
    timer_id: Arc<Mutex<Option<i32>>>,
    #[allow(unused)]
    js_closure: Closure<dyn FnMut()>
}

impl Timer {
    pub fn new<F>(mut cb: F) -> Timer where F: FnMut() + 'static {
        let window = web_sys::window().unwrap(); // XXX errors
        let timer_id = Arc::new(Mutex::new(None));
        let timer_id2 = timer_id.clone();
        let js_closure = Closure::wrap(Box::new(move || { 
            *lock!(timer_id2) = None;
            cb()
        }) as Box<dyn FnMut()>);
        Timer { window, timer_id, js_closure }
    }

    pub fn stop(&mut self) {
        if let Some(timer_id) = lock!(self.timer_id).as_ref() {
            self.window.clear_timeout_with_handle(*timer_id);
        }
        *lock!(self.timer_id) = None;
    }

    pub fn go(&mut self, interval: i32) {
        self.stop();
        *lock!(self.timer_id) = Some(self.window.set_timeout_with_callback_and_timeout_and_arguments_0(&self.js_closure.as_ref().unchecked_ref(),interval).ok().unwrap());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}
