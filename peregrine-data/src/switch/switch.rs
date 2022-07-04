use std::collections::HashMap;
use super::switchoverlay::SwitchOverlay;
use super::trackconfig::TrackConfigNode;
use crate::switch::track::Track;

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    radio: bool,
    value: bool,
    tracks: Vec<Track>,
    triggers: Vec<Track>
}

impl Switch {
    pub(super) fn new() -> Switch {
        Switch {
            kids: HashMap::new(),
            value: false,
            radio: false,
            tracks: vec![],
            triggers: vec![]
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

    pub(super) fn set(&mut self, yn: bool) {
        self.value = yn;
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
            kid.value = false;
        }
    }

    pub(super) fn get_triggered(&self, out: &mut Vec<Track>) {
        if !self.value { return; }
        out.extend(self.triggers.iter().cloned());
        for kid in self.kids.values() {
            kid.get_triggered(out);
        }
    }

    pub(super) fn build_track_config_list<'a>(&'a self, want_track: &Track, out: &mut TrackConfigNode, path: &mut Vec<&'a str>,mut active: bool, overlay: &SwitchOverlay) {
        if self.tracks.contains(want_track) { active = true; }
        if active { out.add_path(path); }
        let kids = self.kids.iter();
        for (kid_name,kid) in kids {
            path.push(kid_name);
            if kid.value {
                kid.build_track_config_list(want_track,out,path,active,overlay);
            }
            path.pop();
        }
    }
}
