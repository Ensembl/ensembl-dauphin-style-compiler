use std::collections::{ HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ KeyboardEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ EventSystem };
use super::lowlevel::Modifiers;
use js_sys::Date;
use super::mapping::InputMap;

enum KeyboardEventKind {
    Down,
    Up
}

pub(super) struct KeyboardEventHandler {
    distributor: Distributor<InputEvent>,
    mapping: InputMap,
    current: HashSet<InputEventKind>,
    modifiers: Arc<Mutex<Modifiers>>
}

impl KeyboardEventHandler {
    fn new(distributor: &Distributor<InputEvent>, mapping: &InputMap, modifiers: &Arc<Mutex<Modifiers>>) -> KeyboardEventHandler {
        KeyboardEventHandler {
            distributor: distributor.clone(),
            mapping: mapping.clone(),
            current: HashSet::new(),
            modifiers: modifiers.clone()
        }
    }

    fn abandon(&mut self, _event: &Event) {
        for kind in self.current.drain() {
            self.distributor.send(InputEvent {
                details: kind,
                start: false,
                amount: vec![],
                timestamp_ms: Date::now()
            })
        }
    }

    fn keyboard_event(&mut self, event_kind: &KeyboardEventKind, event: &KeyboardEvent) {
        let modifiers = Modifiers {
            shift: event.shift_key(),
            control: event.ctrl_key() || event.meta_key(),
            alt: event.alt_key()
        };
        *self.modifiers.lock().unwrap() = modifiers.clone();
        if let Some((kind,args)) = self.mapping.map(&event.key(),&modifiers) {
            let down = match event_kind {
                KeyboardEventKind::Down => true,
                KeyboardEventKind::Up => false
            };
            if self.current.contains(&kind) != down { // ie not a repeat
                if down { self.current.insert(kind.clone()); } else { self.current.remove(&kind); }
                self.distributor.send(InputEvent {
                    details: kind,
                    start: down,
                    amount: args,
                    timestamp_ms: Date::now()
                })
            }
            event.stop_propagation();
            event.prevent_default();
        }
    }
}

pub(super) fn keyboard_events(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &InputMap, modifiers: &Arc<Mutex<Modifiers>>) -> Result<EventSystem<KeyboardEventHandler>,Message> {
    let body = dom.body();
    let mut events = EventSystem::new(KeyboardEventHandler::new(distributor,mapping,modifiers));
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
