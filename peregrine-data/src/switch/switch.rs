use std::collections::HashMap;
use peregrine_toolkit::eachorevery::eoestruct::{StructBuilt, StructTemplate};
use super::switchoverlay::SwitchOverlay;
use super::trackconfig::TrackConfigNode;
use crate::switch::track::Track;

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    radio: bool,
    value: StructBuilt,
    tracks: Vec<Track>,
    triggers: Vec<Track>,
    null: StructBuilt, // Convenience
}

impl Switch {
    pub(super) fn new() -> Switch {
        let null = StructTemplate::new_null().build().ok().unwrap();
        Switch {
            kids: HashMap::new(),
            value: null.clone(),
            radio: false,
            tracks: vec![],
            triggers: vec![],
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

    pub(super) fn set(&mut self, value: StructBuilt) {
        self.value = value;
    }

    pub(super) fn set_radio(&mut self, yn: bool) { self.radio = yn; }

    pub(super) fn clear_if_radio(&mut self) {
        if self.radio {
            self.unset_kids();
        }
    }

    pub fn add_track(&mut self, track: &Track, trigger: bool) {
        self.tracks.push(track.clone());
        if trigger {
            self.triggers.push(track.clone());
        }
    }

    fn unset_kids(&mut self) {
        for kid in self.kids.values_mut() {
            kid.value = self.null.clone();
        }
    }

    pub(super) fn get_triggered(&self, out: &mut Vec<Track>) {
        if !self.value.truthy() { return; }
        out.extend(self.triggers.iter().cloned());
        for kid in self.kids.values() {
            kid.get_triggered(out);
        }
    }

    pub(super) fn build_track_config_list<'a>(&'a self, want_track: &Track, out: &mut TrackConfigNode, path: &mut Vec<&'a str>,mut active: bool, overlay: &SwitchOverlay) {
        if self.tracks.contains(want_track) { active = true; }
        if active { out.add_path(path,self.value.clone()); }
        let kids = self.kids.iter();
        for (kid_name,kid) in kids {
            path.push(kid_name);
            if kid.value.truthy() {
                kid.build_track_config_list(want_track,out,path,active,overlay);
            }
            path.pop();
        }
    }
}
