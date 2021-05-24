use std::collections::{ HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ KeyboardEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ EventSystem };
use super::lowlevel::{ Modifiers, LowLevelState };
use js_sys::Date;
use super::mapping::InputMap;

enum KeyboardEventKind {
    Down,
    Up
}

pub(super) struct KeyboardEventHandler {
    state: LowLevelState,
    current: HashSet<InputEventKind>
}

impl KeyboardEventHandler {
    fn new(state: &LowLevelState) -> KeyboardEventHandler {
        KeyboardEventHandler {
            state: state.clone(),
            current: HashSet::new(),
        }
    }

    fn abandon(&mut self, _event: &Event) {
        for kind in self.current.drain() {
            self.state.send(kind,false,&[]);
        }
    }

    fn keyboard_event(&mut self, event_kind: &KeyboardEventKind, event: &KeyboardEvent) {
        let modifiers = Modifiers {
            shift: event.shift_key(),
            control: event.ctrl_key() || event.meta_key(),
            alt: event.alt_key()
        };
        self.state.update_modifiers(modifiers.clone());
        let down = match event_kind {
            KeyboardEventKind::Down => true,
            KeyboardEventKind::Up => false
        };
        if let Some((kind,args)) = self.state.map(&event.key(),&modifiers) {
            if self.current.contains(&kind) != down { // ie not a repeat
                if down { self.current.insert(kind.clone()); } else { self.current.remove(&kind); }
                self.state.send(kind,down,&args);
            }
            event.stop_propagation();
            event.prevent_default();
        }
    }
}

pub(super) fn keyboard_events(state: &LowLevelState) -> Result<EventSystem<KeyboardEventHandler>,Message> {
    let body = state.dom().body();
    let mut events = EventSystem::new(KeyboardEventHandler::new(state));
    events.add(body,"keyup",|handler,event| {
        handler.keyboard_event(&KeyboardEventKind::Up,event);
    })?;
    events.add(body,"keydown",|handler,event| {
        handler.keyboard_event(&KeyboardEventKind::Down,event);
    })?;
    events.add_window("blur",|handler,event| {
        handler.abandon(event);
    })?;
    Ok(events)
}
