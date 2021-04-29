use std::{collections::hash_map::DefaultHasher, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use super::switch::Switch;
use super::track::Track;

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

    fn list_configs(&self, out: &mut Vec<Vec<String>>, path: &mut Vec<String>) {
        for (kid_name,kid) in self.kids.iter() {
            path.push(kid_name.to_string());
            out.push(path.to_vec());
            kid.list_configs(out,path);
            path.pop();
        }
    }
}

fn hashmap_hasher<H: Hasher, K: Hash+PartialEq+Eq+PartialOrd+Ord, V: Hash>(map: &HashMap<K,V>, state: &mut H) {
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
        let mut kids : Vec<_> = self.kids.keys().collect();
        hashmap_hasher(&self.kids,state);
    }
}

#[derive(Clone)]
pub struct TrackConfig {
    track: Track,
    hash: u64,
    values: Arc<TrackConfigNode>
}

impl TrackConfig {
    fn new(track: &Track, root: TrackConfigNode) -> TrackConfig {
        let mut state = DefaultHasher::new();
        root.hash_value().hash(&mut state);
        track.hash(&mut state);
        TrackConfig {
            track: track.clone(),
            hash: state.finish(),
            values: Arc::new(root)
        }
    }

    pub fn track(&self) -> &Track { &self.track }
    pub fn contains(&self, path: &[&str]) -> bool { self.values.contains(path) }
    pub fn get(&self, path: &[&str]) -> Option<Vec<String>> { self.values.get(path) }

    fn list_configs(&self, out: &mut Vec<Vec<String>>) {
        self.values.list_configs(out,&mut vec![]);
    }
}

impl fmt::Debug for TrackConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut track_config_list = vec![];
        self.list_configs(&mut track_config_list);
        let track_config_list : Vec<_> = track_config_list.iter().map(|x| {
            x.join(".")
        }).collect();
        let track_config_list = track_config_list.join(";");
        write!(f,"{:?}({}) ",self.track,&track_config_list)?;
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

#[derive(Clone)]
pub struct TrackConfigList {
    configs: Arc<HashMap<Track,Arc<TrackConfig>>>,
    hash: u64
}

impl Hash for TrackConfigList {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for TrackConfigList {
    fn eq(&self, other: &TrackConfigList) -> bool {
        self.hash == other.hash
    }
}

impl Eq for TrackConfigList {}

impl fmt::Debug for TrackConfigList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (track_name,track_config) in self.configs.iter() {
            track_config.fmt(f)?;
        }
        Ok(())
    }
}

// XXX list split to new file

impl TrackConfigList {
    pub(crate) fn new(root: &Switch) -> TrackConfigList {
        let mut triggered = vec![];
        root.get_triggered(&mut triggered);
        let mut builder = HashMap::new();
        for track in triggered {
            builder.insert(track.clone(),TrackConfigNode::new());
        }
        let mut path = vec![];
        root.build_track_config_list(&mut builder,&mut path,&[]);
        let builder = builder.drain().map(|(track,v)| { 
            (track.clone(),TrackConfig::new(&track,v))
        });
        let builder = builder.map(|(k,v)| (k,Arc::new(v))).collect();
        let mut hasher = DefaultHasher::new();
        hashmap_hasher(&builder,&mut hasher);
        TrackConfigList {
            configs: Arc::new(builder),
            hash: hasher.finish()
        }
    }

    pub(crate) fn get_track(&self, track: &Track) -> Option<Arc<TrackConfig>> {
        self.configs.get(track).cloned()
    }

    pub(crate) fn list_tracks(&self) -> Vec<Track> {
        self.configs.keys().cloned().collect()
    }
}
