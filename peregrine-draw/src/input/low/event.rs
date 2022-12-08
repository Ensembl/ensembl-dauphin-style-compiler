use crate::util::error::{ confused_browser, confused_browser_option };
use web_sys::{ HtmlElement, window, Window };
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use crate::Message;
use std::sync::{ Arc, Mutex };
use wasm_bindgen::JsValue;
use std::convert::{ TryInto, TryFrom };

fn window_catch() -> Result<Window,Message> {
    confused_browser_option(window(),"could not get window object")
}

fn make_closure2<F,T>(mut cb: F) -> Closure<dyn FnMut(JsValue)>
        where F: FnMut(&T) + 'static, T: TryFrom<JsValue> {
    Closure::wrap(Box::new(move |event: JsValue| {
        if let Ok(event) = event.try_into() {
            cb(&event);
        }
    }) as Box<dyn FnMut(JsValue)>)
}

struct CallbackStore {
    closure: Closure<dyn FnMut(JsValue)>,
    finisher: Option<Box<dyn FnOnce(&mut CallbackStore) -> Result<(),Message>>>
}

impl Drop for CallbackStore {
    fn drop(&mut self) {
        if let Some(finisher) = self.finisher.take() {
            finisher(self);
        }
    }
}

#[derive(Clone)]
pub(crate) struct EventHandle(Arc<Mutex<CallbackStore>>);

impl EventHandle {
    pub(crate) fn new<F,T>(element: &HtmlElement, name: &str, cb: F) -> Result<EventHandle,Message>
            where F: FnMut(&T) + 'static, T: TryFrom<JsValue> + 'static {
        let closure = make_closure2(cb);
        let element = element.clone();
        let name = name.to_string();
        confused_browser(element.add_event_listener_with_callback(&name,closure.as_ref().unchecked_ref()))?;
        Ok(EventHandle(Arc::new(Mutex::new(CallbackStore {
            closure,
            finisher: Some(Box::new(move |this| {
                confused_browser(element.remove_event_listener_with_callback(&name,this.closure.as_ref().unchecked_ref()))
            }))
        }))))
    }

    pub(crate) fn new_window<F,T>(name: &str, cb: F) -> Result<EventHandle,Message>
            where F: FnMut(&T) + 'static, T: TryFrom<JsValue> + 'static {
        let closure = make_closure2(cb);
        let name = name.to_string();
        confused_browser(window_catch()?.add_event_listener_with_callback(&name,closure.as_ref().unchecked_ref()))?;
        Ok(EventHandle(Arc::new(Mutex::new(CallbackStore {
            closure,
            finisher: Some(Box::new(move |this| {
                confused_browser(window_catch()?.remove_event_listener_with_callback(&name,this.closure.as_ref().unchecked_ref()))
            }))
        }))))
    }
}
