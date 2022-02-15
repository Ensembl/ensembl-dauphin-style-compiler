use std::sync::{ Arc, Mutex };
use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;
use web_sys::{ CustomEvent, CustomEventInit, Window };
use wasm_bindgen::JsCast;
use crate::util::message::{ Message, message };
use std::thread_local;
use keyed::{ KeyedOptionalValues, keyed_handle };

use super::timer::Timer;

lazy_static! {
    static ref IDENTITY : Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

const MESSAGE_KEY : &str = "domutil-bell";

keyed_handle!(ObjectKey);

struct Objects {
    event: CustomEvent,
    window: Window
}

thread_local! {
    static OBJECTS : Arc<Mutex<KeyedOptionalValues<ObjectKey,Objects>>> = Arc::new(Mutex::new(KeyedOptionalValues::new()));
}

#[derive(Clone)]
pub struct BellSender {
    identity: u64,
    event: ObjectKey
}

fn make_event(identity: u64) -> Result<ObjectKey,Message> {
    let name = &format!("{}-{}",MESSAGE_KEY,identity);
    let mut cvi = CustomEventInit::new();
    cvi.bubbles(false);
    let event = CustomEvent::new_with_event_init_dict(name,&cvi).map_err(|e| Message::ConfusedWebBrowser(format!("cannot create event: {:?}",e)))?;
    let window = web_sys::window().ok_or( Message::ConfusedWebBrowser(format!("cannot get window object")))?;
    Ok(OBJECTS.with(|events| {
        let mut locked = events.lock().unwrap();
        locked.add(Objects { window, event })
    }))
}

impl Drop for BellSender {
    fn drop(&mut self) {
        OBJECTS.with(|events| {
            let mut locked = events.lock().unwrap();
            locked.remove(&self.event);
        });
    }
}

impl BellSender {
    fn new(identity: u64) -> Result<BellSender,Message> {
        Ok(BellSender {
            event: make_event(identity)?,
            identity
        })
    }

    pub fn ring(&self) -> Result<(),Message> {
        OBJECTS.with(|objects| {
            let locked = objects.lock().unwrap();
            let object = locked.get(&self.event).unwrap();
            object.window.dispatch_event(&object.event).map_err(|e| Message::ConfusedWebBrowser(format!("cannot send event: {:?}",e)))
        })?;
        Ok(())
    }
}

struct BellReceiverState {
    callbacks: Arc<Mutex<Vec<Box<dyn Fn() + 'static>>>>,
    name: String,
    closure: Closure<dyn FnMut()>
}

fn run_callbacks(callbacks: &Arc<Mutex<Vec<Box<dyn Fn()>>>>) {
    for cb in callbacks.lock().unwrap().iter_mut() {
        (cb)();
    }
}

fn pg_window() -> Result<Window,Message> {
    web_sys::window().ok_or( Message::ConfusedWebBrowser(format!("cannot get window object")))
}

impl BellReceiverState {
    fn new(identity: u64) -> Result<BellReceiverState,Message> {
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        let callbacks2 = callbacks.clone();
        let mut timer = Timer::new(move || { run_callbacks(&callbacks2); });
        let closure =  Closure::wrap(Box::new(move || {
            timer.go(0);
        }) as Box<dyn FnMut()>);
        let name = format!("{}-{}",MESSAGE_KEY,identity);
        let out = BellReceiverState {
            name: name.clone(),
            callbacks,
            closure
        };
        let window = pg_window()?;
        window.add_event_listener_with_callback(&name,out.closure.as_ref().unchecked_ref()).map_err(|e| Message::ConfusedWebBrowser(format!("cannt create event callback: {:?}",e.as_string())))?;
        Ok(out)
    }

    fn add(&mut self, callback: Box<dyn Fn() + 'static>) {
        self.callbacks.lock().unwrap().push(callback);
    }
}

impl Drop for BellReceiverState {
    fn drop(&mut self) {        
        if let Some(window) = pg_window().ok() {
            window.remove_event_listener_with_callback(&self.name,self.closure.as_ref().unchecked_ref()).unwrap_or(());
        }

    }
}

#[derive(Clone)]
pub struct BellReceiver(Arc<Mutex<BellReceiverState>>);

impl BellReceiver {
    fn new(identity: u64) -> Result<BellReceiver,Message> {
        Ok(BellReceiver(Arc::new(Mutex::new(BellReceiverState::new(identity)?))))
    }

    pub fn add<T>(&mut self, callback: T) where T: Fn() + 'static {
        self.0.lock().unwrap().add(Box::new(callback));
    }
}

pub fn make_bell() -> Result<(BellSender,BellReceiver),Message> {
    let mut source = IDENTITY.lock().unwrap();
    let identity = *source;
    *source += 1;
    drop(source);
    Ok((BellSender::new(identity)?,BellReceiver::new(identity)?))
}
