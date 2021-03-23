use anyhow::{ self, Context, anyhow as err };
use std::sync::{ Arc, Mutex };
use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;
use web_sys::{ CustomEvent, CustomEventInit, HtmlElement };
use crate::util::safeelement::SafeElement;
use wasm_bindgen::JsCast;
use crate::util::message::{ Message, message };

pub fn js_panic(e: Result<(),Message>) {
    match e {
        Ok(_) => (),
        Err(e) => {
            message(e);
        }
    }
}

lazy_static! {
    static ref IDENTITY : Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

const MESSAGE_KEY : &str = "domutil-bell";

#[derive(Clone)]
pub struct BellSender {
    el: SafeElement,
    identity: u64
}

impl BellSender {
    fn new(identity: u64, el: &HtmlElement) -> BellSender {
        BellSender {
            el: SafeElement::new(el),
            identity
        }
    }

    pub fn ring(&self) -> Result<(),Message> {
        let name = &format!("{}-{}",MESSAGE_KEY,self.identity);
        let mut cvi = CustomEventInit::new();
        cvi.bubbles(false);
        let e = CustomEvent::new_with_event_init_dict(name,&cvi).map_err(|e| Message::ConfusedWebBrowser(format!("cannot create event: {:?}",e)))?;
        self.el.get()?.dispatch_event(&e).map_err(|e| Message::ConfusedWebBrowser(format!("cannot send event: {:?}",e)))?;
        Ok(())
    }
}

struct BellReceiverState {
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + 'static>>>>,
    name: String,
    el: HtmlElement,
    closure: Option<Closure<dyn Fn()>>
}

fn run_callbacks(callbacks: Arc<Mutex<Vec<Box<dyn Fn()>>>>) {
    for cb in callbacks.lock().unwrap().iter_mut() {
        (cb)();
    }
}

fn add_zero_callback(closure: JsValue) -> Result<(),Message> {
    let window = web_sys::window().ok_or( Message::ConfusedWebBrowser(format!("cannot get window object")))?;
    window.set_timeout_with_callback_and_timeout_and_arguments_0(&closure.into(),0)
        .map_err(|_| Message::ConfusedWebBrowser(format!("cannot set zero timeout")))?;
    Ok(())
}

impl BellReceiverState {
    fn new(identity: u64, el: &HtmlElement) -> Result<BellReceiverState,Message> {
        let mut out = BellReceiverState {
            name: format!("{}-{}",MESSAGE_KEY,identity),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            el: el.clone(),
            closure: None
        };
        out.call_dom()?;
        Ok(out)
    }

    fn add(&mut self, callback: Box<dyn Fn() + 'static>) {
        self.callbacks.lock().unwrap().push(callback);
    }

    fn call_dom(&mut self) -> Result<(),Message> {
        let callbacks = self.callbacks.clone();
        self.closure = Some(Closure::wrap(Box::new(move || {
            let callbacks = callbacks.clone();
            let closure = Closure::once_into_js(move || {
                run_callbacks(callbacks);
            });
            js_panic(add_zero_callback(closure));
        })));
        self.el.add_event_listener_with_callback(&self.name,self.closure.as_ref().unwrap().as_ref().unchecked_ref()).map_err(|e| Message::ConfusedWebBrowser(format!("cannt create event callback")))?;
        Ok(())
    }
}

impl Drop for BellReceiverState {
    fn drop(&mut self) {
        if let Some(closure) = self.closure.take() {
            self.el.remove_event_listener_with_callback(&self.name,closure.as_ref().unchecked_ref()).unwrap_or(());
        }
    }
}

#[derive(Clone)]
pub struct BellReceiver(Arc<Mutex<BellReceiverState>>);

impl BellReceiver {
    fn new(identity: u64, el: &HtmlElement) -> Result<BellReceiver,Message> {
        Ok(BellReceiver(Arc::new(Mutex::new(BellReceiverState::new(identity,el)?))))
    }

    pub fn add<T>(&mut self, callback: T) where T: Fn() + 'static {
        self.0.lock().unwrap().add(Box::new(callback));
    }
}

pub fn make_bell(el: &HtmlElement) -> Result<(BellSender,BellReceiver),Message> {
    let mut source = IDENTITY.lock().unwrap();
    let identity = *source;
    *source += 1;
    drop(source);
    Ok((BellSender::new(identity,el),BellReceiver::new(identity,el)?))
}