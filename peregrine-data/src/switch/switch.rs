use std::collections::HashMap;
use peregrine_toolkit::eachorevery::eoestruct::{StructBuilt, StructTemplate};
use super::expansion::Expansion;
use super::switchoverlay::SwitchOverlay;
use super::trackconfig::TrackConfigNode;
use crate::switch::track::Track;

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    radio: bool,
    value: StructBuilt,
    tracks: Vec<Track>,
    triggers: Vec<Track>,
    expansions: Vec<Expansion>,
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

    // TODO expended expansions
    pub(super) fn find_expansions(&mut self, path: &[&str]) -> Vec<(Expansion,String)> {
        if path.len() > 0 {
            if self.expansions.len() > 0 {
                self.expansions.iter().map(|x| (x.clone(),path[0].to_string())).collect()
            } else {
                self.get_or_make(&path[0]).find_expansions(&path[1..])
            }
        } else {
            vec![]
        }
    }

    pub(super) fn remove(&mut self, path: &[&str]) {
        if path.len() > 1 {
            if let Some(child) = self.kids.get_mut(&path[0].to_string()) {
                child.remove(&path[1..]);
            }
        } else if path.len() > 0 {
            self.kids.remove(&path[0].to_string());
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

    pub(crate) fn add_track(&mut self, track: &Track, trigger: bool) {
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

    pub(super) fn get_triggered(&self, out: &mut Vec<Track>) {
        if !self.value.truthy() { return; }
        out.extend(self.triggers.iter().cloned());
        for kid in self.kids.values() {
            kid.get_triggered(out);
        }
    }

    pub(super) fn build_track_config<'a>(&'a self, want_track: &Track, out: &mut TrackConfigNode, path: &mut Vec<&'a str>,mut active: bool, overlay: &SwitchOverlay, also_kids: bool) {
        if self.tracks.contains(want_track) { active = true; }
        if active { out.add_path(path,self.value.clone()); }
        let kids = self.kids.iter();
        if also_kids {
            for (kid_name,kid) in kids {
                path.push(kid_name);
                kid.build_track_config(want_track,out,path,active,overlay,kid.value.truthy());
                path.pop();
            }
        }
    }
}
