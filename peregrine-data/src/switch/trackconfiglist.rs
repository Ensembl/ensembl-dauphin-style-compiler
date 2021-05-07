use std::{collections::hash_map::DefaultHasher, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use super::switch::Switch;
use crate::switch::allotment::AllotmentList;
use super::track::Track;
use super::trackconfig::{ TrackConfig, TrackConfigNode, hashmap_hasher };
use crate::core::{ Layout, Scale };

#[derive(Clone)]
pub struct TrackConfigList {
    configs: Arc<HashMap<Track,Arc<TrackConfig>>>,
    allotments: AllotmentList,
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
        for (_,track_config) in self.configs.iter() {
            track_config.fmt(f)?;
        }
        Ok(())
    }
}

impl TrackConfigList {
    pub(crate) fn new(root: &Switch) -> TrackConfigList {
        let mut triggered = vec![];
        root.get_triggered(&mut triggered);
        let mut builder = HashMap::new();
        let mut allotments = vec![];
        for track in triggered {
            builder.insert(track.clone(),TrackConfigNode::new());
            allotments.extend(track.allotments().iter().cloned());
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
            allotments: AllotmentList::new(allotments),
            hash: hasher.finish()
        }
    }

    pub(crate) fn get_track(&self, track: &Track) -> Option<Arc<TrackConfig>> {
        self.configs.get(track).cloned()
    }

    pub(crate) fn list_tracks(&self) -> Vec<Track> {
        self.configs.keys().cloned().collect()
    }

    fn new_filter<F>(&self, f: F) -> TrackConfigList where F: Fn(&Track) -> bool {
        let mut builder = HashMap::new();
        for (track,config) in self.configs.iter() {
            if f(track) {
                builder.insert(track.clone(),config.clone());
            }
        }
        let mut hasher = DefaultHasher::new();
        hashmap_hasher(&builder,&mut hasher);
        let a : Vec<_> = self.allotments.allotments().iter().map(|x| format!("{:?}",x)).collect();
        use web_sys::console;
        console::log_1(&format!("new allotments {:?}",a).into());
        TrackConfigList {
            configs: Arc::new(builder),
            allotments: self.allotments.clone(),
            hash: hasher.finish()
        }
    }
}

#[derive(Clone)]
pub struct TrainTrackConfigList(TrackConfigList);

impl TrainTrackConfigList {
    pub fn new(layout: &Layout, scale: &Scale) -> TrainTrackConfigList {
        TrainTrackConfigList(layout.track_config_list().new_filter(|track| {
            track.available(layout,scale)
        }))
    }

    pub(crate) fn get_track(&self, track: &Track) -> Option<Arc<TrackConfig>> {
        self.0.get_track(track)
    }

    pub(crate) fn list_tracks(&self) -> Vec<Track> {
        self.0.list_tracks()
    }
}
