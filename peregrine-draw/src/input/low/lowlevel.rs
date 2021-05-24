use std::sync::{ Arc, Mutex };
use crate::input::InputEventKind;
use crate::{PeregrineDom, PgCommanderWeb, run::PgPeregrineConfig};
use crate::util::Message;
use super::{event::EventSystem, keyboardinput::{KeyboardEventHandler, keyboard_events}, mouseinput::mouse_events};
use super::mapping::{ InputMapBuilder };
use super::mouseinput::{ MouseEventHandler };
use crate::input::{ InputEvent, Distributor };
use super::mapping::InputMap;
use js_sys::Date;
use commander::cdr_timer;
use super::pointer::cursor::{ Cursor, CursorHandle };
use crate::run::CursorCircumstance;

#[derive(Debug,Clone,Hash,PartialEq,Eq)]

pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool
}

// XXX pub
#[derive(Clone)]
pub struct LowLevelState {
    commander: PgCommanderWeb,
    distributor: Distributor<InputEvent>,
    dom: PeregrineDom,
    mapping: InputMap,
    modifiers: Arc<Mutex<Modifiers>>,
    cursor: Cursor
}

impl LowLevelState {
    fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, config: &PgPeregrineConfig) -> Result<(LowLevelState,Distributor<InputEvent>),Message> {
        let mut mapping = InputMapBuilder::new();
        mapping.add_config(config)?;
        let modifiers = Arc::new(Mutex::new(Modifiers {
            shift: false,
            control: false,
            alt: false
        }));
        let distributor = Distributor::new();
        Ok((LowLevelState {
            cursor: Cursor::new(dom,config)?,
            dom: dom.clone(),
            commander: commander.clone(),
            distributor: distributor.clone(),
            mapping: mapping.build(),
            modifiers,
        },distributor))
    }

    pub(super) fn update_modifiers(&self, modifiers: Modifiers) {
        *self.modifiers.lock().unwrap() = modifiers;
    }

    pub(super) fn map(&self, key: &str, modifiers: &Modifiers) -> Option<(InputEventKind,Vec<f64>)> {
        self.mapping.map(key,modifiers)
    }

    pub(super) fn dom(&self) -> &PeregrineDom { &self.dom }
    pub(super) fn modifiers(&self) -> Modifiers { self.modifiers.lock().unwrap().clone() }

    pub fn send(&self, kind: InputEventKind, start: bool, args: &[f64]) {
        self.distributor.send(InputEvent {
            details: kind,
            start,
            amount: args.to_vec(),
            timestamp_ms: Date::now()
        })
    }

    pub(super) fn timer<F>(&self, timeout: f64, cb: F) where F: FnOnce() + 'static {
        self.commander.add::<()>("hold-timer", 50, None, None, Box::pin(async move {
            cdr_timer(timeout).await;
            cb();
            Ok(())
        }));
    }

    pub fn set_cursor(&self, circ: &CursorCircumstance) -> CursorHandle {
        self.cursor.set(circ)
    }
}

#[derive(Clone)]
pub struct LowLevelInput {
    keyboard: EventSystem<KeyboardEventHandler>,
    mouse: EventSystem<MouseEventHandler>,
    distributor: Distributor<InputEvent>
}

impl LowLevelInput {
    pub fn new(dom: &PeregrineDom, commander: &PgCommanderWeb, config: &PgPeregrineConfig) -> Result<LowLevelInput,Message> {
        let (state,distributor) = LowLevelState::new(dom,commander,config)?;
        let keyboard = keyboard_events(&state)?;
        let mouse = mouse_events(config,&state)?;
        let cursor = Cursor::new(dom,config)?;
        Ok(LowLevelInput { keyboard, mouse, distributor })
    }

    pub fn distributor_mut(&mut self) -> &mut Distributor<InputEvent> { &mut self.distributor }
}
