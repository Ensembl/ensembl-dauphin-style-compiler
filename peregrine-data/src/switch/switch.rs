use std::hash::{ Hash, Hasher };
use std::collections::hash_map::DefaultHasher;
use std::sync::{ Arc, Mutex };
use std::collections::{ HashMap, HashSet };
use super::trackconfig::TrackConfigNode;
use super::trackconfiglist::TrackConfigList;
use crate::switch::track::Track;

fn hash_path(data: &[&str]) -> u64 {
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish()
}

#[derive(Debug)]
pub(crate) struct SwitchOverlay {
    full_set: Vec<Vec<String>>,
    set: HashSet<u64>,
    clear: HashSet<u64>
}

impl SwitchOverlay {
    pub(crate) fn new() -> SwitchOverlay {
        SwitchOverlay {
            full_set: vec![],
            set: HashSet::new(),
            clear: HashSet::new()
        }
    }

    pub(crate) fn set(&mut self, path: &[&str]) {
        for i in 0..path.len() {
            self.set.insert(hash_path(&path[0..i]));
        }
        self.full_set.push(path.iter().map(|x| x.to_string()).collect());
    }

    pub(crate) fn clear(&mut self, path: &[&str]) {
        self.clear.insert(hash_path(path));
    }

    pub fn apply(&self, path: &[&str]) -> Option<bool> {
        let h = hash_path(path);
        if self.set.contains(&h) { return Some(true); }
        if self.clear.contains(&h) { return Some(false); }
        None
    }

    pub(super) fn add_set(&self, node: &mut TrackConfigNode) {
        for path in &self.full_set {
            node.add_path(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
        }
    }
}

pub(crate) struct Switch {
    kids: HashMap<String,Switch>,
    radio: bool,
    set: bool,
    tracks: Vec<Track>,
    triggers: Vec<Track>
}

impl Switch {
    fn new() -> Switch {
        Switch {
            kids: HashMap::new(),
            set: false,
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

    fn get_target(&mut self, path: &[&str]) -> &mut Switch {
        if path.len() > 0 {
            self.get_or_make(&path[0]).get_target(&path[1..])
        } else {
            self
        }
    }

    fn unset_kids(&mut self) {
        for kid in self.kids.values_mut() {
            kid.set = false;
        }
    }

    pub(super) fn get_triggered(&self, out: &mut Vec<Track>) {
        if !self.set { return; }
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
            if overlay.apply(path).unwrap_or(kid.set) {
                kid.build_track_config_list(want_track,out,path,active,overlay);
            }
            path.pop();
        }
    }
}

pub(super) struct SwitchesData {
    root: Switch,
    track_config_list: Option<TrackConfigList>
}

impl SwitchesData {
    fn get_track_config_list(&mut self) -> &TrackConfigList {
        if self.track_config_list.is_none() {
            self.track_config_list = Some(TrackConfigList::new(&self));
        }
        self.track_config_list.as_ref().unwrap()
    }

    pub(super) fn get_triggered(&self) -> Vec<Track> {
        let mut triggered = vec![];
        self.root.get_triggered(&mut triggered);
        triggered
    }

    pub(super) fn build_track_config_list(&self, track: &Track) -> TrackConfigNode {
        let mut out = TrackConfigNode::new();
        let overlay = track.overlay();
        self.root.build_track_config_list(track, &mut out, &mut vec![], false,&overlay);
        overlay.add_set(&mut out);
        out
    }
}

#[derive(Clone)]
pub struct Switches(Arc<Mutex<SwitchesData>>);

impl Switches {
    pub fn new() -> Switches {
        let out = Switches(Arc::new(Mutex::new(SwitchesData {
            root: Switch::new(),
            track_config_list: None
        })));
        out.set_switch(&[]);
        out
    }

    pub fn set_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();
        if path.len() > 0 {
            let parent = data.root.get_target(&path[0..(path.len()-1)]);
            if parent.radio {
                parent.unset_kids();
            }
        }
        let target = data.root.get_target(path);
        target.set = true;
        data.track_config_list = None;
    }

    pub fn clear_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();
        let target = data.root.get_target(path);
        target.set = false;
        data.track_config_list = None;
    }

    pub fn radio_switch(&self, path: &[&str], yn: bool) {
        let mut data = self.0.lock().unwrap();
        data.root.get_target(path).radio = yn;        
        data.track_config_list = None;
    }

    pub fn add_track(&self, path: &[&str], track: &Track, trigger: bool) {
        let mut data = self.0.lock().unwrap();
        let target = data.root.get_target(path);
        target.tracks.push(track.clone());
        if trigger {
            target.triggers.push(track.clone());
        }
        data.track_config_list = None;
    }

    pub fn get_track_config_list(&self) -> TrackConfigList {
        let mut data = self.0.lock().unwrap();
        data.get_track_config_list().clone()
    }
}
