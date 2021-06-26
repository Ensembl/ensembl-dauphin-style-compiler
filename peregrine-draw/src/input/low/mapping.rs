use std::{collections::{ HashMap }};
use std::sync::{ Arc };
use crate::input::{InputEventKind, low::modifiers::Modifiers};
use crate::util::{ Message };
use crate::run::PgPeregrineConfig;
use crate::run::{ PgConfigKey };

use super::{keyspec::parse_keyspec, modifiers::ModifiersPattern};

/* Mappings are space separated alternatives for an action. Each alternative is a hyphen-separated
 * list. The last element must be the generated mapped unicode codepoint, if any, or from the standard special
 * values (with " " replaced by Space). These can be modified by the initial values which are one of shift,
 * control (synonym ctrl), alt. Modifiers are case-insensitive. Examples Ctrl-W, ArrowDown, Ctrl-Alt-Shift--
 */

pub struct InputMapBuilder {
    mapping: HashMap<String,Vec<(ModifiersPattern,InputEventKind,Vec<f64>)>>
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
        for (key,modifiers,args) in parse_keyspec(keys)? {
            self.mapping.entry(key).or_insert_with(|| vec![]).push((modifiers,kind.clone(),args));
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
    pub fn map(&self, key: &str, modifiers: &Modifiers) -> Vec<(InputEventKind,Vec<f64>)> {
        let mut out = vec![];
        for (modifiers_pattern,kind,args) in self.0.mapping.get(key).unwrap_or(&vec![]) {
            if modifiers_pattern.is_match(modifiers) {
                out.push((kind.clone(),args.clone()));
            }
        }
        out
    }
}
