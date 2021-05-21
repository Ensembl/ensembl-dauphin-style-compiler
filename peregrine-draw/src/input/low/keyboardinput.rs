use std::collections::{ HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ KeyboardEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ add_event, remove_event, window_add_event, window_remove_event };
use super::lowlevel::Modifiers;
use js_sys::Date;
use super::mapping::InputMap;

enum KeyboardEventKind {
    Down,
    Up
}

struct KeyboardEventHandler {
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
        }
    }
}

#[derive(Clone)]
pub struct KeyboardInput {
    down_closure: Arc<Closure<dyn Fn(KeyboardEvent) + 'static>>,
    up_closure: Arc<Closure<dyn Fn(KeyboardEvent) + 'static>>,
    blur_closure: Arc<Closure<dyn Fn(Event) + 'static>>,
    el: HtmlElement
}

fn make_keyboard_event(kind: KeyboardEventKind, handler: &Arc<Mutex<KeyboardEventHandler>>) -> Closure<dyn Fn(KeyboardEvent)> {
    let handler = handler.clone();
    Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let handler = handler.clone();
        handler.lock().unwrap().keyboard_event(&kind,&event);
    }) as Box<dyn Fn(KeyboardEvent)>)
}

fn make_event(handler: &Arc<Mutex<KeyboardEventHandler>>) -> Closure<dyn Fn(Event)> {
    let handler = handler.clone();
    Closure::wrap(Box::new(move |event: Event| {
        let handler = handler.clone();
        handler.lock().unwrap().abandon(&event);
    }) as Box<dyn Fn(Event)>)
}

impl KeyboardInput {
    pub fn new(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &InputMap, modifiers: &Arc<Mutex<Modifiers>>) -> Result<KeyboardInput,Message> {
        let body = dom.body();
        let handler = Arc::new(Mutex::new(KeyboardEventHandler::new(distributor,mapping,modifiers)));
        let up_closure = make_keyboard_event(KeyboardEventKind::Up,&handler);
        let down_closure = make_keyboard_event(KeyboardEventKind::Down,&handler);
        let blur_closure = make_event(&handler);
        add_event(body,"keydown",&down_closure)?;
        add_event(body,"keyup",&up_closure)?;
        window_add_event("blur",&blur_closure)?;
        Ok(KeyboardInput {
            up_closure: Arc::new(up_closure),
            down_closure: Arc::new(down_closure),
            blur_closure: Arc::new(blur_closure),
            el: body.clone()
        })
    }

    pub fn finish(&self) -> Result<(),Message> { // XXX pub
        remove_event(&self.el,"keydown",&self.down_closure.as_ref())?;
        remove_event(&self.el,"keyup",&self.up_closure.as_ref())?;
        window_remove_event("blur",&self.blur_closure.as_ref())?;
        Ok(())
    }
}

impl Drop for KeyboardInput {
    fn drop(&mut self) {
        self.finish().ok();
    }
}
