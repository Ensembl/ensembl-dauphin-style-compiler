use std::{collections::{hash_map::DefaultHasher, BTreeMap}, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use peregrine_toolkit::{eachorevery::eoestruct::{StructBuilt }};

use super::{track::Track, switches::SwitchesData};

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
    values: Arc<BTreeMap<String,StructBuilt>>
}

impl TrackConfig {
    pub(super) fn new(track: &Track, switches_data: &SwitchesData) -> TrackConfig {
        let program = track.program();
        let mapping = track.mapping();
        let mut values = mapping.apply(switches_data);
        program.apply_defaults(&mut values);
        let mut state = DefaultHasher::new();
        values.hash(&mut state);
        track.hash(&mut state);
        TrackConfig {
            track: track.clone(),
            hash: state.finish(),
            values: Arc::new(values)
        }
    }

    pub fn track(&self) -> &Track { &self.track }
    pub fn value(&self, name: &str) -> Option<&StructBuilt> { self.values.get(name) }
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
