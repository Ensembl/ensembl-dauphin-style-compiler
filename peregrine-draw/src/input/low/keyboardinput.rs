use std::collections::{ HashSet, HashMap };
use std::mem;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use web_sys::{ KeyboardEvent, Event };
use crate::input::{ InputEventKind };
use crate::util::{ Message };
use super::event::{ EventHandle };
use super::lowlevel::{ LowLevelState };
use super::modifiers::{KeyboardModifiers, Modifiers};

enum KeyboardEventKind { Down, Up }

struct KeyboardEventHandlerState {
    down_character: HashMap<String,(String,Modifiers)>,
    current: HashSet<InputEventKind>    
}

#[derive(Clone)]
pub(super) struct KeyboardEventHandler {
    low_level: LowLevelState,
    state: Arc<Mutex<KeyboardEventHandlerState>>
}

impl KeyboardEventHandler {
    fn new(low_level: &LowLevelState) -> KeyboardEventHandler {
        KeyboardEventHandler {
            low_level: low_level.clone(),
            state: Arc::new(Mutex::new(KeyboardEventHandlerState{
                down_character: HashMap::new(),
                current: HashSet::new(),    
            }))
        }
    }

    fn abandon(&mut self, _event: &Event) {
        let current = mem::replace(&mut lock!(self.state).current,HashSet::new());
        for kind in current {
            self.low_level.send(kind,false,&[]);
        }
    }

    fn keyboard_event(&mut self, event_kind: &KeyboardEventKind, event: &KeyboardEvent) {
        self.low_level.update_keyboard_modifiers(KeyboardModifiers::new(
            event.shift_key(),
            event.ctrl_key() || event.meta_key(),
            event.alt_key()
        ));
        let mut modifiers = self.low_level.modifiers();
        let mut key = event.key();
        let mut state = lock!(self.state);
        let down = match event_kind {
            KeyboardEventKind::Down => {
                if let Some((down_key,down_modifiers)) = state.down_character.get(&event.code()) {
                    modifiers = down_modifiers.clone();
                    key = down_key.to_string();
                } else {
                    state.down_character.insert(event.code(),(event.key(),modifiers.clone()));
                }
                true
            }
            KeyboardEventKind::Up => {
                if let Some((down_key,down_modifiers)) = state.down_character.remove(&event.code()) {
                    modifiers = down_modifiers.clone();
                    key = down_key.to_string();
                }
                false
            }
        };
        drop(state);
        for (kind,args)in self.low_level.map(&key,&modifiers) {
            let mut state = lock!(self.state);
            let repeat = state.current.contains(&kind) == down;
            if !repeat {
                if down { state.current.insert(kind.clone()); } else { state.current.remove(&kind); }
            }
            drop(state);
            if !repeat {
                self.low_level.send(kind,down,&args);
            }
            event.stop_propagation();
            event.prevent_default();
        }
    }
}

pub(super) fn keyboard_events(state: &LowLevelState) -> Result<Vec<EventHandle>,Message> {
    let body = state.dom().body();
    let mut handles = vec![];
    let handler =KeyboardEventHandler::new(state);
    let mut handler2 = handler.clone();
    handles.push(EventHandle::new(body,"keyup",move |event| {
        handler2.keyboard_event(&KeyboardEventKind::Up,event);
    })?);
    let mut handler2 = handler.clone();
    handles.push(EventHandle::new(body,"keydown",move |event| {
        handler2.keyboard_event(&KeyboardEventKind::Down,event);
    })?);
    let mut handler2 = handler.clone();
    handles.push(EventHandle::new_window("blur",move|event| {
        handler2.abandon(event);
    })?);
    Ok(handles)
}
