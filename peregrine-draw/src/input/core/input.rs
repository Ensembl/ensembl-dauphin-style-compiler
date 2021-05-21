use std::sync::{ Arc, Mutex };
use crate::{PeregrineAPI, PeregrineDom, run::PgPeregrineConfig, PgCommanderWeb };
use crate::util::Message;
use crate::input::low::lowlevel::LowLevelInput;
use crate::input::translate::Physics;
use crate::input::translate::debug::debug_register;

// XXX to  util
#[derive(Clone)]
pub struct Distributor<T>(Arc<Mutex<Vec<Box<dyn Fn(&T) + 'static>>>>);

impl<T> Distributor<T> {
    pub fn new() -> Distributor<T> {
        Distributor(Arc::new(Mutex::new(vec![])))
    }

    pub fn add<F>(&mut self, cb: F) where F: Fn(&T) + 'static {
        self.0.lock().unwrap().push(Box::new(cb));
    }

    pub fn send(&self, value: T) {
        let streams = self.0.lock().unwrap();
        for stream in streams.iter() {
            stream(&value);
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub enum InputEventKind {
    PullLeft,
    PullRight,
    PullIn,
    PullOut,
    PixelsLeft, // [pixels]
    PixelsRight, // [pixels]
    PixelsScale, // [multiplier,screen-prop]
    DebugAction
}

impl InputEventKind {
    pub fn each() -> Vec<InputEventKind> {
        vec![
            InputEventKind::PullLeft,
            InputEventKind::PullRight,
            InputEventKind::PullIn,
            InputEventKind::PullOut,
            InputEventKind::PixelsLeft,
            InputEventKind::PixelsRight,
            InputEventKind::PixelsScale,
            InputEventKind::DebugAction
        ]
    }

}

#[derive(Debug,Clone)]
pub struct InputEvent {
    pub details: InputEventKind,
    pub start: bool,
    pub amount: Vec<f64>,
    pub timestamp_ms: f64
}

#[derive(Clone)]
pub struct Input {
    low_level: LowLevelInput
}

impl Input {
    pub fn new(dom: &PeregrineDom, config: &PgPeregrineConfig, api: &PeregrineAPI, commander: &PgCommanderWeb) -> Result<Input,Message> {
        let mut low_level = LowLevelInput::new(dom,config)?;
        Physics::new(config,&mut low_level,api,commander)?;
        debug_register(config,&mut low_level,api)?;
        Ok(Input {
            low_level
        })
    }
}
