use std::{collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet}, hash::{ Hash, Hasher }};
use std::fmt;
use std::sync::{ Arc };
use std::collections::HashMap;
use eachorevery::eoestruct::StructValue;
use peregrine_toolkit::{error::Error};
use peregrine_toolkit_async::sync::{asynconce::AsyncOnce};
use super::{track::Track, switches::{SwitchesData}};
use super::trackconfig::{ TrackConfig, hashmap_hasher };
use crate::{core::{ Layout, Scale }, PgDauphin, TrackModel};

#[derive(Clone)]
pub(crate) struct TrackConfigListBuilder(AsyncOnce<Result<TrackConfigList,Error>>,u64);

impl TrackConfigListBuilder {
    pub(super) fn new(switches_data: &SwitchesData, pgd: &PgDauphin) -> TrackConfigListBuilder {
        let builder = switches_data.get_triggered().iter().map(|model| {
            let settings = model.mapping().apply(switches_data);
            (model.clone(),settings)
        }).collect::<BTreeSet<_>>();
        let mut hasher = DefaultHasher::new();
        builder.hash(&mut hasher);
        let hash = hasher.finish();
        let pgd = pgd.clone();
        TrackConfigListBuilder(AsyncOnce::new(Box::pin(async move {
            TrackConfigList::new(builder,hash,&pgd).await
        })),hash)
    }

    pub async fn track_config_list(&self) -> Result<TrackConfigList,Error> { self.0.get().await }
}

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

#[cfg(debug_assertions)]
impl fmt::Debug for TrackConfigList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut configs = self.configs.iter().collect::<Vec<_>>();
        configs.sort_by_cached_key(|k| k.0.id());
        for (_,track_config) in configs.iter() {
            track_config.fmt(f)?;
        }
        Ok(())
    }
}

impl TrackConfigList {
    async fn new(models: BTreeSet<(TrackModel,(BTreeMap<String,StructValue>,BTreeMap<String,Vec<String>>))>, hash: u64, pgd: &PgDauphin) -> Result<TrackConfigList,Error> {
        let mut configs = HashMap::new();
        for (model, settings) in models.iter() {
            let mut settings = settings.clone();
            let track = model.to_track(&pgd).await?;
            track.program().apply_defaults(&mut settings.0);
            configs.insert(track.clone(),Arc::new(TrackConfig::new(&track,settings.0,&settings.1)));
        }
        Ok(TrackConfigList {
            configs: Arc::new(configs),
            hash
        })
    }

    pub(crate) fn compatible_with(&self, builder: &TrackConfigListBuilder) -> bool {
        self.hash == builder.1
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
        TrackConfigList {
            configs: Arc::new(builder),
            hash: hasher.finish()
        }
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq)]
pub struct TrainTrackConfigList(TrackConfigList);

impl TrainTrackConfigList {
    pub fn new(layout: &Layout, scale: &Scale) -> TrainTrackConfigList {
        TrainTrackConfigList(layout.track_config_list().new_filter(|track| {
            track.available(layout,scale)
        }))
    }

    pub(crate) fn list_tracks(&self) -> Vec<Track> {
        self.0.list_tracks()
    }
}
