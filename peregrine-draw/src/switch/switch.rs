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
}

#[derive(Clone)]
pub struct Switches {
    root: Arc<Mutex<Switch>>
}

impl Switches {
    pub fn new() -> Switches {
        Switches {
            root: Arc::new(Mutex::new(Switch::new()))
        }
    }

    pub fn set_switch(&self, path: &[&str]) {
        self.root.lock().unwrap().get_target(path).set = true;
    }

    pub fn clear_switch(&self, path: &[&str]) {
        self.root.lock().unwrap().get_target(path).set = false;        
    }

    pub fn add_track(&self, path: &[&str], track: &str) {
        self.root.lock().unwrap().get_target(path).tracks.push(track.to_string());
    }

    pub fn get_tracks(&self) -> Vec<String> {
        let mut tracks = vec![];
        self.root.lock().unwrap().get_tracks(&mut tracks);
        tracks
    }
}
