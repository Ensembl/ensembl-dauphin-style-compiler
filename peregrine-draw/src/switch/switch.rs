use std::{collections::hash_map::DefaultHasher, hash::{ Hash, Hasher }};
use std::sync::{ Arc, Mutex };
use std::collections::HashMap;

struct Switch {
    kids: HashMap<String,Switch>,
    set: bool,
    tracks: Vec<String>
}

impl Switch {
    fn new() -> Switch {
        Switch {
            kids: HashMap::new(),
            set: false,
            tracks: vec![]
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

    fn get_tracks(&self, tracks: &mut Vec<String>) {
        if !self.set { return; }
        tracks.extend(self.tracks.iter().cloned());
        for (_,kid) in self.kids.iter() {
            kid.get_tracks(tracks);
        }
    }

    fn set_hash<H: Hasher>(&self, hasher: &mut H) {
        if !self.set { return; }
        let mut kids : Vec<String> = self.kids.keys().cloned().collect();
        kids.len().hash(hasher);
        kids.sort();
        for name in kids {
            let kid = self.kids.get(&name).unwrap();
            if kid.set {
                name.hash(hasher);
                kid.set_hash(hasher);
            }
        }
    }
}

struct SwitchesData {
    root: Switch,
    track_paths: HashMap<String,Vec<Vec<String>>>,
    track_hash_cache: HashMap<String,u64>
}

impl SwitchesData {
    fn compute_track_hash(&mut self, track: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        if let Some(paths) =  self.track_paths.get(track) {
            let mut paths = paths.clone();
            paths.sort();
            for path in paths {
                path.hash(&mut hasher);
                self.root.get_target(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>()).set_hash(&mut hasher);
            }
        }
        hasher.finish()
    }
}

#[derive(Clone)]
pub struct Switches(Arc<Mutex<SwitchesData>>);

impl Switches {
    pub fn new() -> Switches {
        let out = Switches(Arc::new(Mutex::new(SwitchesData {
            root: Switch::new(),
            track_paths: HashMap::new(),
            track_hash_cache: HashMap::new()
        })));
        out.set_switch(&[]);
        out
    }

    pub fn set_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();        
        data.root.get_target(path).set = true;
        data.track_hash_cache.clear();
    }

    pub fn clear_switch(&self, path: &[&str]) {
        let mut data = self.0.lock().unwrap();
        data.root.get_target(path).set = false;        
        data.track_hash_cache.clear();
    }

    pub fn add_track(&self, path: &[&str], track: &str) {
        let mut data = self.0.lock().unwrap();
        data.root.get_target(path).tracks.push(track.to_string());
        let paths = data.track_paths.entry(track.to_string()).or_insert_with(||vec![]);
        paths.push(path.iter().map(|x| x.to_string()).collect());
        data.track_hash_cache.clear();
    }

    pub fn get_tracks(&self) -> Vec<String> {
        let mut tracks = vec![];
        self.0.lock().unwrap().root.get_tracks(&mut tracks);
        tracks
    }

    fn get_track_hash(&self, track: &str) -> u64 {
        let mut data = self.0.lock().unwrap();
        if !data.track_hash_cache.contains_key(track) {
            let hash = data.compute_track_hash(track);
            data.track_hash_cache.insert(track.to_string(),hash);
        }
        *data.track_hash_cache.get(track).unwrap()
    }
}
