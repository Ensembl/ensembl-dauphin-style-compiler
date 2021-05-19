use std::collections::{ HashMap, HashSet };
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use wasm_bindgen::prelude::*;
use web_sys::{ KeyboardEvent, HtmlElement };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ add_event, remove_event };
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::{ Key, Modifiers };
use js_sys::Date;

/* Keyboard mappings are space separated alternatives for an action. Each alternative is a hyphen-separated
 * list. The last element must be the generated mapped unicode codepoint, if any, or from the standard special
 * values (with " " replaced by Space). These can be modified by the initial values which are one of shift,
 * control (synonym ctrl), alt. Modifiers are case-insensitive. Examples Ctrl-W, ArrowDown, Ctrl-Alt-Shift--
 */

fn parse_one(spec: &str) -> Option<Key> {
    let mut spec = spec.to_string();
    let mut trailing_minus = false;
    if spec.ends_with('-') {
        spec.pop();
        trailing_minus = true;
    }
    let mut parts = spec.split('-').collect::<Vec<_>>();
    let text = if trailing_minus {
        Some("-")
    } else {
        parts.pop()
    };
    let mut shift = false;
    let mut control = false;
    let mut alt = false;
    for modifier in parts {
        let modifier = modifier.to_lowercase();
        if modifier == "shift" { shift = true; }
        if modifier == "ctrl" || modifier == "control" { control = true; }
        if modifier == "alt" { alt = true; }
    }
    if let Some(text) = text {
        Some(Key {
            text: text.to_string(),
            modifiers: Modifiers { shift, control, alt }
        })
    } else {
        None
    }
}

fn parse_keyspec(spec: &str) -> Vec<Key> {
    spec.split_whitespace().filter_map(|spec| {
        parse_one(spec)
    }).collect()
}

pub struct KeyboardMapBuilder {
    mapping: HashMap<Key,InputEventKind>
}

#[derive(Clone)]
pub struct KeyboardMap(Arc<KeyboardMapBuilder>);

impl KeyboardMapBuilder {
    pub(super) fn new() -> KeyboardMapBuilder {
        KeyboardMapBuilder {
            mapping: HashMap::new()
        }
    }

    pub(super) fn add_mapping(&mut self, keys: &str, kind: InputEventKind) {
        for key in parse_keyspec(keys) {
            self.mapping.insert(key,kind.clone());
        }
    }

    pub(super) fn add_config(&mut self, config: &PgPeregrineConfig) -> Result<(),Message> {
        for kind in InputEventKind::each() {
            self.add_mapping(config.get_str(&PgConfigKey::KeyBindings(kind.clone()))?,kind);
        }
        Ok(())
    }

    pub(super) fn build(self) -> KeyboardMap { KeyboardMap(Arc::new(self)) }
}

impl KeyboardMap {
    fn map(&self, key: &Key) -> Option<InputEventKind> {
        self.0.mapping.get(&key).cloned()
    }
}

struct KeyboardEventHandler {
    distributor: Distributor<InputEvent>,
    mapping: KeyboardMap,
    current: HashSet<InputEventKind>
}

impl KeyboardEventHandler {
    fn new(distributor: &Distributor<InputEvent>, mapping: &KeyboardMap) -> KeyboardEventHandler {
        KeyboardEventHandler {
            distributor: distributor.clone(),
            mapping: mapping.clone(),
            current: HashSet::new()
        }
    }

    fn keyboard_event(&mut self,down: bool, event: &KeyboardEvent) {
        let key = Key {
            text: event.key().to_string(),
            modifiers: Modifiers {
                shift: event.shift_key(),
                control: event.ctrl_key() || event.meta_key(),
                alt: event.alt_key()
            }
        };
        if let Some(kind) = self.mapping.map(&key) {
            if self.current.contains(&kind) != down { // ie not a repeat
                if down { self.current.insert(kind.clone()); } else { self.current.remove(&kind); }
                self.distributor.send(InputEvent {
                    details: kind,
                    start: down,
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
    el: HtmlElement
}

fn make_event(down: bool, handler: &Arc<Mutex<KeyboardEventHandler>>) -> Closure<dyn Fn(KeyboardEvent)> {
    let handler = handler.clone();
    Closure::wrap(Box::new(move |event: KeyboardEvent| {
        let handler = handler.clone();
        handler.lock().unwrap().keyboard_event(down,&event);
    }) as Box<dyn Fn(KeyboardEvent)>)
}

impl KeyboardInput {
    pub fn new(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &KeyboardMap) -> Result<KeyboardInput,Message> {
        let body = dom.body();
        let handler = Arc::new(Mutex::new(KeyboardEventHandler::new(distributor,mapping)));
        let up_closure = make_event(false,&handler);
        let down_closure = make_event(true,&handler);
        add_event(body,"keydown",&down_closure)?;
        add_event(body,"keyup",&up_closure)?;
        Ok(KeyboardInput {
            up_closure: Arc::new(up_closure),
            down_closure: Arc::new(down_closure),
            el: body.clone()
        })
    }

    pub fn finish(&self) -> Result<(),Message> { // XXX pub
        remove_event(&self.el,"keydown",&self.down_closure.as_ref())?;
        remove_event(&self.el,"keyup",&self.up_closure.as_ref())?;
        Ok(())
    }
}

impl Drop for KeyboardInput {
    fn drop(&mut self) {
        self.finish().ok();
    }
}
