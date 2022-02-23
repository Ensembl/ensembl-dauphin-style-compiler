use std::sync::{Arc, Mutex};
use js_sys::Math;
use lazy_static::lazy_static;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{CustomEventInit, CustomEvent};

const PREFIX : &str = "pgcustom";

lazy_static! {
    static ref SERIAL : Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

fn crypto_random_u64(bytes: &mut [u8;8]) -> bool {
    let window = web_sys::window().unwrap();
    let crypto = if let Ok(x) = window.crypto() { x } else { return false; };
    if crypto.get_random_values_with_u8_array(bytes).is_err() { return false; }
    true
}

fn random_u64() -> u64 {
    let bytes = &mut [0;8];
    if !crypto_random_u64(bytes) {
        for i in 0..8 {
            bytes[i] = (Math::random()*255.).floor() as u8;
        }
    }
    u64::from_le_bytes(*bytes)
}

fn random_name() -> String {
    format!("{}-{}",PREFIX,random_u64())
}

#[derive(Clone)]
pub struct CustomSender {
    name: String,
}

impl CustomSender {
    pub fn new(name: &str) -> CustomSender {
        CustomSender {
            name: name.to_string()
        }
    }

    pub fn dispatch(&self) {
        let mut cvi = CustomEventInit::new();
        cvi.bubbles(false);
        let event = CustomEvent::new_with_event_init_dict(&self.name,&cvi).unwrap();
        let window = web_sys::window().unwrap();
        window.dispatch_event(&event).ok();
    }
}

pub struct Custom {
    name: String,
    closure: Closure<dyn FnMut()>
}

impl Custom {
    pub fn new<F>(mut cb: F) -> Custom where F: FnMut() + 'static {
        let name = random_name();
        let closure =  Closure::wrap(Box::new(move || cb()) as Box<dyn FnMut()>);
        let window = web_sys::window().unwrap();
        window.add_event_listener_with_callback(&name,closure.as_ref().unchecked_ref()).ok();
        Custom {
            name,
            closure
        }
    }

    pub fn sender(&self) -> CustomSender {
        CustomSender::new(&self.name)
    }
}

impl Drop for Custom {
    fn drop(&mut self) {
        let window = web_sys::window().unwrap();
        window.remove_event_listener_with_callback(&self.name,self.closure.as_ref().unchecked_ref()).ok();
    }
}
