use crate::util::error::{ confused_browser, confused_browser_option };
use peregrine_toolkit::lock;
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

fn make_closure<H,F,T>(handler: &Arc<Mutex<H>>, cb: F) -> Closure<dyn Fn(JsValue)>
        where F: Fn(&mut H,&T) + 'static, H: 'static, T: TryFrom<JsValue> {
    let handler = handler.clone();
    Closure::wrap(Box::new(move |event: JsValue| {
        let handler = handler.clone();
        if let Ok(event) = event.try_into() {
            cb(&mut handler.lock().unwrap(),&event);
        }
    }) as Box<dyn Fn(JsValue)>)
}

struct CallbackStore {
    closure: Closure<dyn Fn(JsValue)>,
    finisher: Option<Box<dyn FnOnce(&mut CallbackStore) -> Result<(),Message>>>
}

impl CallbackStore {
    fn new<H,F,T>(handler: &Arc<Mutex<H>>, element: &HtmlElement, name: &str, cb: F) -> Result<CallbackStore,Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static, H: 'static {
        let closure = make_closure(handler,cb);
        let element = element.clone();
        let name = name.to_string();
        confused_browser(element.add_event_listener_with_callback(&name,closure.as_ref().unchecked_ref()))?;
        Ok(CallbackStore {
            closure,
            finisher: Some(Box::new(move |this| {
                confused_browser(element.remove_event_listener_with_callback(&name,this.closure.as_ref().unchecked_ref()))
            }))
        })
    }

    fn new_window<H,F,T>(handler: &Arc<Mutex<H>>, name: &str, cb: F) -> Result<CallbackStore,Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static, H: 'static {
        let closure = make_closure(handler,cb);
        let name = name.to_string();
        confused_browser(window_catch()?.add_event_listener_with_callback(&name,closure.as_ref().unchecked_ref()))?;
        Ok(CallbackStore {
            closure,
            finisher: Some(Box::new(move |this| {
                confused_browser(window_catch()?.remove_event_listener_with_callback(&name,this.closure.as_ref().unchecked_ref()))
            }))
        })
    }

    fn finish(&mut self) -> Result<(),Message> {
        if let Some(finisher) = self.finisher.take() {
            finisher(self)?
        }
        Ok(())
    }
}

pub struct EventSystemImpl<H> where H: 'static {
    handler: Arc<Mutex<H>>,
    js_handlers: Vec<CallbackStore>
}

impl<H> EventSystemImpl<H> {
    fn new(handler: H) -> EventSystemImpl<H> {
        EventSystemImpl {
            handler: Arc::new(Mutex::new(handler)),
            js_handlers: vec![],
        }
    }

    fn add<F,T>(&mut self, element: &HtmlElement, name: &str, cb: F) -> Result<(),Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static {
        self.js_handlers.push( CallbackStore::new(&self.handler,element,name,cb)?);
        Ok(())
    }

    fn add_window<F,T>(&mut self, name: &str, cb: F) -> Result<(),Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static {
        self.js_handlers.push(CallbackStore::new_window(&self.handler,name,cb)?);
        Ok(())
    }

    fn finish(&mut self) -> Result<(),Message> {
        for mut callback in self.js_handlers.drain(..) {
            callback.finish()?;
        }
        Ok(())
    }
}

impl<H> Drop for EventSystemImpl<H> {
    fn drop(&mut self) {
        self.finish().ok();
    }
}

pub struct EventSystem<H: 'static>(Arc<Mutex<EventSystemImpl<H>>>);

impl<H> Clone for EventSystem<H> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<H: 'static> EventSystem<H> {
    pub fn new(handler: H) -> EventSystem<H> {
        EventSystem(Arc::new(Mutex::new(EventSystemImpl::new(handler))))
    }

    pub fn add<F,T>(&mut self, element: &HtmlElement, name: &str, cb: F) -> Result<(),Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static {
        lock!(self.0).add(element,name,cb)
    }

    pub fn add_window<F,T>(&mut self, name: &str, cb: F) -> Result<(),Message>
                where F: Fn(&mut H,&T) + 'static, T: TryFrom<JsValue> + 'static {
        lock!(self.0).add_window(name,cb)
    }
}