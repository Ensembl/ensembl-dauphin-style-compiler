use std::{collections::{ HashMap, HashSet }, num::ParseFloatError};
use std::sync::{ Arc, Mutex };
use crate::PeregrineDom;
use peregrine_config::ConfigError;
use wasm_bindgen::prelude::*;
use web_sys::{ KeyboardEvent, HtmlElement, Event };
use crate::input::{ InputEvent, InputEventKind, Distributor };
use crate::util::{ Message };
use super::event::{ add_event, remove_event, window_add_event, window_remove_event };
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::{ Key, Modifiers };
use js_sys::Date;
use peregrine_data::DataMessage;

/* Keyboard mappings are space separated alternatives for an action. Each alternative is a hyphen-separated
 * list. The last element must be the generated mapped unicode codepoint, if any, or from the standard special
 * values (with " " replaced by Space). These can be modified by the initial values which are one of shift,
 * control (synonym ctrl), alt. Modifiers are case-insensitive. Examples Ctrl-W, ArrowDown, Ctrl-Alt-Shift--
 */

fn handle_err(v: &str,e: Result<f64,ParseFloatError>) -> Result<f64,Message> {
    e.map_err(|e| Message::DataError(DataMessage::ConfigError(
        ConfigError::BadConfigValue(format!("mapping parsing: {}",v),e.to_string())
    )))
}

fn parse_args(args: &str) -> Result<Vec<f64>,Message> {
    args.split(",").map(|x| handle_err(x,x.trim().parse::<f64>())).collect()
}

fn parse_final_part(text: &str) -> Result<(Vec<f64>,String),Message> {
    let mut args = vec![];
    let mut text = text.to_string();
    if text.ends_with("]") {
        if let Some(start) = text.rfind("[") {
            if start > 0 {
                args = parse_args(&text[(start+1)..(text.len()-1)])?;
                text = text[0..start].to_string();
            }
        }
    }
    Ok((args,text))
}

fn parse_one(spec: &str) -> Result<Option<(Key,Vec<f64>)>,Message> {
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
    if let Some((args,text)) = text.map(|t| parse_final_part(t)).transpose()? {
        Ok(Some((Key {
            text: text.to_string(),
            modifiers: Modifiers { shift, control, alt }
        },args)))
    } else {
        Ok(None)
    }
}

fn parse_keyspec(spec: &str) -> Result<Vec<(Key,Vec<f64>)>,Message> {
    spec.split_whitespace().filter_map(|spec| {
        parse_one(spec).transpose()
    }).collect::<Result<Vec<_>,_>>()
}

pub struct KeyboardMapBuilder {
    mapping: HashMap<Key,(InputEventKind,Vec<f64>)>
}

#[derive(Clone)]
pub struct KeyboardMap(Arc<KeyboardMapBuilder>);

impl KeyboardMapBuilder {
    pub(super) fn new() -> KeyboardMapBuilder {
        KeyboardMapBuilder {
            mapping: HashMap::new()
        }
    }

    pub(super) fn add_mapping(&mut self, keys: &str, kind: InputEventKind) -> Result<(),Message> {
        for (key,args) in parse_keyspec(keys)? {
            self.mapping.insert(key,(kind.clone(),args));
        }
        Ok(())
    }

    pub(super) fn add_config(&mut self, config: &PgPeregrineConfig) -> Result<(),Message> {
        for kind in InputEventKind::each() {
            if let Some(spec) = config.try_get_str(&PgConfigKey::KeyBindings(kind.clone())) {
                self.add_mapping(spec,kind)?;
            }
        }
        Ok(())
    }

    pub(super) fn build(self) -> KeyboardMap { KeyboardMap(Arc::new(self)) }
}

impl KeyboardMap {
    fn map(&self, key: &Key) -> Option<(InputEventKind,Vec<f64>)> {
        self.0.mapping.get(&key).cloned()
    }
}

enum KeyboardEventKind {
    Down,
    Up
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

    fn abandon(&mut self, event: &Event) {
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
        let key = Key {
            text: event.key().to_string(),
            modifiers: Modifiers {
                shift: event.shift_key(),
                control: event.ctrl_key() || event.meta_key(),
                alt: event.alt_key()
            }
        };
        if let Some((kind,args)) = self.mapping.map(&key) {
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
    pub fn new(distributor: &Distributor<InputEvent>, dom: &PeregrineDom, mapping: &KeyboardMap) -> Result<KeyboardInput,Message> {
        let body = dom.body();
        let handler = Arc::new(Mutex::new(KeyboardEventHandler::new(distributor,mapping)));
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
