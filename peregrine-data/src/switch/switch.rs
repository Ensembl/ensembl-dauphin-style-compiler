use std::collections::{HashMap, HashSet};
use peregrine_toolkit::{eachorevery::eoestruct::StructValue};
use super::expansion::Expansion;
use crate::TrackModel;

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    radio: bool,
    value: StructValue,
    tracks: Vec<TrackModel>,
    triggers: Vec<TrackModel>,
    expansions: Vec<Expansion>,
    null: StructValue // Convenience
}

impl Switch {
    pub(super) fn new() -> Switch {
        let null = StructValue::new_null();
        Switch {
            kids: HashMap::new(),
            value: null.clone(),
            radio: false,
            tracks: vec![],
            triggers: vec![],
            expansions: vec![],
            null
        }
    }

    fn get_or_make(&mut self, name: &str) -> &mut Switch {
        self.kids.entry(name.to_string()).or_insert_with(|| {
            Switch::new()
        })
    }

    pub(super) fn get_target(&mut self, path: &[&str]) -> &mut Switch {
        if path.len() > 0 {
            self.get_or_make(&path[0]).get_target(&path[1..])
        } else {
            self
        }
    }

    pub(super) fn get_value(&self, path: &[&str]) -> &StructValue {
        if path.len() > 0 {
            if !self.value.truthy() { return &self.null; }
            if let Some(kid) = self.kids.get(&path[0].to_string()) {
                kid.get_value(&path[1..])
            } else {
                &self.null
            }
        } else {
            &self.value
        }
    }

    pub(super) fn find_expansions(&mut self,) -> &[Expansion] { &self.expansions }

    pub(super) fn remove(&mut self, path: &[&str]) {
        if path.len() > 1 {
            if let Some(child) = self.kids.get_mut(&path[0].to_string()) {
                child.remove(&path[1..]);
            }
        } else if path.len() > 0 {
            self.kids.remove(&path[0].to_string());
        }
    }

    pub(super) fn set(&mut self, value: StructValue) {
        self.value = value;
    }

    pub(super) fn set_radio(&mut self, yn: bool) { self.radio = yn; }

    pub(super) fn clear_if_radio(&mut self) {
        if self.radio {
            self.unset_kids();
        }
    }

    pub(crate) fn add_track(&mut self, track: &TrackModel, trigger: bool) {
        self.tracks.push(track.clone());
        if trigger {
            self.triggers.push(track.clone());
        }
    }

    pub(crate) fn add_expansion(&mut self, expansion: &Expansion) {
        self.expansions.push(expansion.clone());
    }

    fn unset_kids(&mut self) {
        for kid in self.kids.values_mut() {
            kid.value = self.null.clone();
        }
    }

    pub(super) fn get_triggered(&self, out: &mut HashSet<TrackModel>) {
        if !self.value.truthy() { return; }
        out.extend(self.triggers.iter().cloned());
        for (name,kid) in self.kids.iter() {
            kid.get_triggered(out);
        }
    }
}
