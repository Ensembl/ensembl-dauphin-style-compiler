use std::{collections::{hash_map::DefaultHasher, BTreeMap}, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use eachorevery::eoestruct::StructValue;

use super::{track::Track};

pub(super) fn hashmap_hasher<H: Hasher, K: Hash+PartialEq+Eq+PartialOrd+Ord, V: Hash>(map: &HashMap<K,V>, state: &mut H) {
    let mut kids : Vec<_> = map.keys().collect();
    kids.sort();
    kids.len().hash(state);
    for kid in kids.drain(..) {
        kid.hash(state);
        map.get(kid).as_ref().unwrap().hash(state);
    }
}

#[derive(Clone)]
pub struct TrackConfig {
    track: Track,
    hash: u64,
    values: Arc<BTreeMap<String,StructValue>>,
    underlying_switch: Arc<BTreeMap<String,Vec<String>>>
}

impl TrackConfig {
    pub(super) fn new(track: &Track, values: BTreeMap<String,StructValue>, underlying_switch: &BTreeMap<String,Vec<String>>) -> TrackConfig {
        let mut state = DefaultHasher::new();
        values.hash(&mut state);
        track.hash(&mut state);
        TrackConfig {
            track: track.clone(),
            hash: state.finish(),
            values: Arc::new(values),
            underlying_switch: Arc::new(underlying_switch.clone())
        }
    }

    pub fn track(&self) -> &Track { &self.track }
    pub fn value(&self, name: &str) -> Option<&StructValue> { self.values.get(name) }
    pub fn underlying_switch(&self, setting: &str) -> Option<&Vec<String>> { self.underlying_switch.get(setting) }
}

#[cfg(debug_assertions)]
impl fmt::Debug for TrackConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let settings = self.values.iter().map(|(key,value)| 
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
