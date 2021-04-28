use std::{collections::hash_map::DefaultHasher, hash::{ Hash, Hasher }};
use std::sync::{ Arc };
use std::collections::HashMap;
use super::switch::Switch;

#[derive(Clone)]
pub(super) struct TrackConfigNode {
    kids: HashMap<String,Box<TrackConfigNode>>
}

impl TrackConfigNode {
    fn new() -> TrackConfigNode {
        TrackConfigNode {
            kids: HashMap::new()
        }
    }

    pub(super) fn merge(&mut self, path: &[String]) {
        if path.len() > 0 {
            self.kids.entry(path[0].to_string()).or_insert_with(|| Box::new(TrackConfigNode::new())).merge(&path[1..]);
        }
    }

    fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn get(&self, path: &[&str]) -> Option<Vec<String>> {
        if path.len() > 0 {
            self.kids.get(path[0]).and_then(|x| x.get(&path[1..]))
        } else {
            Some(self.kids.keys().cloned().collect())
        }
    }

    fn contains(&self, path: &[&str]) -> bool {
        if path.len() > 0 {
            self.kids.get(path[0]).map(|x| x.contains(&path[1..])).unwrap_or(false)
        } else {
            true
        }
    }
}

impl Hash for TrackConfigNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut kids : Vec<_> = self.kids.keys().collect();
        kids.sort();
        kids.len().hash(state);
        for kid in kids.drain(..) {
            kid.hash(state);
            self.kids.get(kid).as_ref().unwrap().hash(state);
        }
    }
}

#[derive(Clone)]
pub(crate) struct TrackConfig {
    name: String,
    hash: u64,
    values: Arc<TrackConfigNode>
}

impl TrackConfig {
    pub(crate) fn contains(&self, path: &[&str]) -> bool { self.values.contains(path) }
    pub(crate) fn get(&self, path: &[&str]) -> Option<Vec<String>> { self.values.get(path) }
}

impl Hash for TrackConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Clone)]
pub struct TrackConfigList(Arc<HashMap<String,Arc<TrackConfig>>>);

impl TrackConfigList {
    pub(crate) fn new(root: &Switch) -> TrackConfigList {
        let mut triggered = vec![];
        root.get_triggered(&mut triggered);
        let mut builder = HashMap::new();
        for track in triggered {
            builder.insert(track,TrackConfigNode::new());
        }
        let mut path = vec![];
        root.build_track_config_list(&mut builder,&mut path,&[]);
        let builder = builder.drain().map(|(k,v)| { 
            (k.clone(),TrackConfig {
                name: k,
                hash: v.hash_value(),
                values: Arc::new(v)
            })
        });
        let builder = builder.map(|(k,v)| (k,Arc::new(v))).collect();
        TrackConfigList(Arc::new(builder))
    }

    pub(crate) fn get_track(&self, track: &str) -> Option<Arc<TrackConfig>> {
        self.0.get(track).cloned()
    }

    pub(crate) fn list_tracks(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
}
