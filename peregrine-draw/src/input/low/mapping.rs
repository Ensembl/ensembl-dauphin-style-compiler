use std::{collections::{ HashMap }, num::ParseFloatError};
use std::sync::{ Arc };
use peregrine_config::ConfigError;
use crate::input::{ InputEventKind };
use crate::util::{ Message };
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };
use super::lowlevel::{ Modifiers };
use peregrine_data::DataMessage;

/* Mappings are space separated alternatives for an action. Each alternative is a hyphen-separated
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

fn parse_one(spec: &str) -> Result<Option<(LowLevelEvent,Vec<f64>)>,Message> {
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
        Ok(Some((LowLevelEvent {
            text: text.to_string(),
            modifiers: Modifiers { shift, control, alt }
        },args)))
    } else {
        Ok(None)
    }
}

fn parse_keyspec(spec: &str) -> Result<Vec<(LowLevelEvent,Vec<f64>)>,Message> {
    spec.split_whitespace().filter_map(|spec| {
        parse_one(spec).transpose()
    }).collect::<Result<Vec<_>,_>>()
}

#[derive(Debug,Clone,Hash,PartialEq,Eq)]
pub struct LowLevelEvent {
    pub text: String,
    pub modifiers: Modifiers
}

pub struct InputMapBuilder {
    mapping: HashMap<LowLevelEvent,(InputEventKind,Vec<f64>)>
}

#[derive(Clone)]
pub struct InputMap(Arc<InputMapBuilder>);

impl InputMapBuilder {
    pub(super) fn new() -> InputMapBuilder {
        InputMapBuilder {
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

    pub(super) fn build(self) -> InputMap { InputMap(Arc::new(self)) }
}

impl InputMap {
    pub fn map(&self, key: &str, modifiers: &Modifiers) -> Option<(InputEventKind,Vec<f64>)> {
        let event = LowLevelEvent {
            text: key.to_string(),
            modifiers: modifiers.clone()
        };
        self.0.mapping.get(&event).cloned()
    }
}
