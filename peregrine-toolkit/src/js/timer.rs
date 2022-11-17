use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Window;

pub struct Timer {
    window: Window,
    timer_id: Option<i32>,
    #[allow(unused)]
    js_closure: Closure<dyn FnMut()>
}

impl Timer {
    pub fn new<F>(mut cb: F) -> Timer where F: FnMut() + 'static {
        let window = web_sys::window().unwrap(); // XXX errors
        let js_closure = Closure::wrap(Box::new(move || { cb() }) as Box<dyn FnMut()>);
        Timer { window, timer_id: None, js_closure }
    }

    pub fn stop(&mut self) {
        if let Some(timer_id) = &self.timer_id {
            self.window.clear_timeout_with_handle(*timer_id);
        }
        self.timer_id = None;
    }

    pub fn go(&mut self, interval: i32) {
        self.stop();
        self.timer_id = Some(self.window.set_timeout_with_callback_and_timeout_and_arguments_0(&self.js_closure.as_ref().unchecked_ref(),interval).ok().unwrap());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        self.stop();
    }
}
