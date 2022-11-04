use std::{collections::{hash_map::DefaultHasher, BTreeMap}, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use peregrine_toolkit::eachorevery::eoestruct::{StructBuilt, StructTemplate };

use super::{track::Track, switches::SwitchesData};

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub(super) struct TrackConfigNode {
    value: StructBuilt,
    value_hash: u64,
    kids: HashMap<String,Box<TrackConfigNode>>
}

impl TrackConfigNode {
    pub(super) fn empty() -> TrackConfigNode {
        Self::new(false)
    }

    fn rehash(&mut self) {
        let mut state = DefaultHasher::new();
        self.value.hash(&mut state);
        self.value_hash = state.finish();
    }

    fn new(yn: bool) -> TrackConfigNode {
        let mut out = TrackConfigNode {
            value: StructTemplate::new_boolean(yn).build().unwrap(),
            value_hash: 0,
            kids: HashMap::new()
        };
        out.rehash();
        out
    }

    pub(super) fn add_path(&mut self, path: &[&str], value: StructBuilt) {
        if path.len() > 0 {
            self.kids.entry(path[0].to_string()).or_insert_with(|| Box::new(TrackConfigNode::new(true))).add_path(&path[1..],value);
        } else {
            self.value = value;
            self.rehash();
        }
    }

    fn hash_value(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn list(&self, path: &[&str]) -> Option<Vec<String>> {
        if path.len() > 0 {
            self.kids.get(path[0]).and_then(|x| x.list(&path[1..]))
        } else {
            Some(self.kids.keys().cloned().collect())
        }
    }
}

pub(super) fn hashmap_hasher<H: Hasher, K: Hash+PartialEq+Eq+PartialOrd+Ord, V: Hash>(map: &HashMap<K,V>, state: &mut H) {
    let mut kids : Vec<_> = map.keys().collect();
    kids.sort();
    kids.len().hash(state);
    for kid in kids.drain(..) {
        kid.hash(state);
        map.get(kid).as_ref().unwrap().hash(state);
    }
}

impl Hash for TrackConfigNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value_hash.hash(state);
        hashmap_hasher(&self.kids,state);
    }
}

#[derive(Clone)]
pub struct TrackConfig {
    track: Track,
    hash: u64,
    values: Arc<TrackConfigNode>,
    values2: Arc<BTreeMap<String,StructBuilt>>
}

impl TrackConfig {
    pub(super) fn new(track: &Track, root: TrackConfigNode, switches_data: &SwitchesData) -> TrackConfig {
        let program = track.program();
        let mapping = track.mapping();
        let mut values2 = mapping.apply(switches_data);
        program.apply_defaults(&mut values2);
        let mut state = DefaultHasher::new();
        root.hash_value().hash(&mut state);
        values2.hash(&mut state);
        track.hash(&mut state);
        TrackConfig {
            track: track.clone(),
            hash: state.finish(),
            values: Arc::new(root),
            values2: Arc::new(values2)
        }
    }

    pub fn track(&self) -> &Track { &self.track }

    pub fn list(&self, path: &[&str]) -> Option<Vec<String>> { self.values.list(path) }
    pub fn value2(&self, name: &str) -> Option<&StructBuilt> { self.values2.get(name) }
}

#[cfg(debug_assertions)]
impl fmt::Debug for TrackConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let settings = self.values2.iter().map(|(key,value)| 
            {
                format!("{}={:?}",key,value)
            }).collect::<Vec<_>>();
        write!(f,"{:?} {}",self.track.id(),settings.join("; "))?;
        Ok(())
    }
}

impl Hash for TrackConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for TrackConfig {
    fn eq(&self, other: &TrackConfig) -> bool {
        self.hash == other.hash
    }
}

impl Eq for TrackConfig {}
