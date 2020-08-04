use anyhow::{ self, Context };
use std::sync::{ Arc, Mutex };
use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;
use web_sys::{ CustomEvent, CustomEventInit, HtmlElement };
use crate::util::error::js_error;
use crate::util::safeelement::SafeElement;
use wasm_bindgen::JsCast;

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

    pub fn ring(&self) -> anyhow::Result<()> {
        let name = &format!("{}-{}",MESSAGE_KEY,self.identity);
        let mut cvi = CustomEventInit::new();
        cvi.bubbles(false);
        let e = js_error(CustomEvent::new_with_event_init_dict(name,&cvi)).context("creating bell event")?;
        js_error(self.el.get()?.dispatch_event(&e)).context("sending bell event")?;
        Ok(())
    }
}

struct BellReceiverState {
    callbacks: Arc<Mutex<Vec<Box<dyn FnMut() + 'static>>>>,
    name: String,
    el: HtmlElement,
    closure: Option<Closure<dyn Fn()>>
}

impl BellReceiverState {
    fn new(identity: u64, el: &HtmlElement) -> anyhow::Result<BellReceiverState> {
        let mut out = BellReceiverState {
            name: format!("{}-{}",MESSAGE_KEY,identity),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            el: el.clone(),
            closure: None
        };
        out.call_dom()?;
        Ok(out)
    }

    fn add(&mut self, callback: Box<dyn FnMut() + 'static>) {
        self.callbacks.lock().unwrap().push(callback);
    }

    fn call_dom(&mut self) -> anyhow::Result<()> {
        let callbacks = self.callbacks.clone();
        self.closure = Some(Closure::wrap(Box::new(move || {
            for cb in callbacks.lock().unwrap().iter_mut() {
                (cb)();
            }
        }) as Box<dyn Fn()>));
        js_error(self.el.add_event_listener_with_callback(&self.name,self.closure.as_ref().unwrap().as_ref().unchecked_ref()))?;
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
    fn new(identity: u64, el: &HtmlElement) -> anyhow::Result<BellReceiver> {
        Ok(BellReceiver(Arc::new(Mutex::new(BellReceiverState::new(identity,el)?))))
    }

    pub fn add<T>(&mut self, callback: T) where T: FnMut() + 'static {
        self.0.lock().unwrap().add(Box::new(callback));
    }
}

pub fn make_bell(el: &HtmlElement) -> anyhow::Result<(BellSender,BellReceiver)> {
    let mut source = IDENTITY.lock().unwrap();
    let identity = *source;
    *source += 1;
    drop(source);
    Ok((BellSender::new(identity,el),BellReceiver::new(identity,el)?))
}