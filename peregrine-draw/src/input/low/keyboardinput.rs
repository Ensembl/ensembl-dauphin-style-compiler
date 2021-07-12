use std::collections::{ HashSet, HashMap };
use web_sys::{ KeyboardEvent, Event };
use crate::input::{ InputEventKind };
use crate::util::{ Message };
use super::event::{ EventSystem };
use super::lowlevel::{ LowLevelState };
use super::modifiers::{KeyboardModifiers, Modifiers};

enum KeyboardEventKind { Down, Up }

pub(super) struct KeyboardEventHandler {
    state: LowLevelState,
    down_character: HashMap<String,(String,Modifiers)>,
    current: HashSet<InputEventKind>
}

impl KeyboardEventHandler {
    fn new(state: &LowLevelState) -> KeyboardEventHandler {
        KeyboardEventHandler {
            state: state.clone(),
            down_character: HashMap::new(),
            current: HashSet::new(),
        }
    }

    fn abandon(&mut self, _event: &Event) {
        for kind in self.current.drain() {
            self.state.send(kind,false,&[]);
        }
    }

    fn keyboard_event(&mut self, event_kind: &KeyboardEventKind, event: &KeyboardEvent) {
        self.state.update_keyboard_modifiers(KeyboardModifiers::new(
            event.shift_key(),
            event.ctrl_key() || event.meta_key(),
            event.alt_key()
        ));
        let mut modifiers = self.state.modifiers();
        let mut key = event.key();
        let down = match event_kind {
            KeyboardEventKind::Down => {
                if let Some((down_key,down_modifiers)) = self.down_character.get(&event.code()) {
                    modifiers = down_modifiers.clone();
                    key = down_key.to_string();
                } else {
                    self.down_character.insert(event.code(),(event.key(),modifiers.clone()));
                }
                true
            }
            KeyboardEventKind::Up => {
                if let Some((down_key,down_modifiers)) = self.down_character.remove(&event.code()) {
                    modifiers = down_modifiers.clone();
                    key = down_key.to_string();
                }
                false
            }
        };
        for (kind,args)in self.state.map(&key,&modifiers) {
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
