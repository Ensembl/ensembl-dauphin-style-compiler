use std::hash::Hash;
use crate::core::{ Scale, StickId };
use crate::switch::trackconfig::TrackConfig;
use crate::switch::track::Track;
use serde_cbor::Value as CborValue;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Region {
    stick: StickId,
    scale: Scale,
    index: u64
}

impl Region {
    pub fn new(stick: &StickId, index: u64, scale: &Scale) -> Region {
        Region { stick: stick.clone(), scale: scale.clone(), index }
    }

    pub fn encode(&self) -> CborValue {
        CborValue::Array(vec![
            self.stick.encode(),
            self.scale.encode(),
            CborValue::Integer(self.index as i128)
        ])
    }

    pub fn stick(&self) -> &StickId { &self.stick }
    pub fn index(&self) -> u64 { self.index }
    pub fn scale(&self) -> &Scale { &self.scale }
    pub fn min_value(&self) -> u64 { self.scale.bp_in_carriage() * self.index }
    pub fn max_value(&self) -> u64 { self.scale.bp_in_carriage() * (self.index+1) }

    pub fn best_region(&self, track: &Track) -> Region {
        if let Some(better_scale) = track.best_scale(&self.scale) {
            let better_index = better_scale.convert_index(&self.scale,self.index);
            Region {
                stick: self.stick.clone(),
                scale: better_scale,
                index: better_index
            }
        } else {
            self.clone()
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeRequestCore {
    region: Region,
    track: TrackConfig
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeRequest {
    core: ShapeRequestCore,
    warm: bool
}

impl Hash for ShapeRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.core.hash(state);
    }
}

impl PartialEq for ShapeRequest {
    fn eq(&self, other: &Self) -> bool {
        self.core == other.core
    }
}

impl Eq for ShapeRequest {}

impl ShapeRequest {
    pub fn new(region: &Region, track: &TrackConfig, warm: bool) -> ShapeRequest {
        ShapeRequest {
            core: ShapeRequestCore {
                region: region.clone(),
                track: track.clone(),
            },
            warm
        }
    }

    pub fn region(&self) -> &Region { &self.core.region }
    pub fn track(&self) -> &TrackConfig { &self.core.track }
    pub fn warm(&self) -> bool { self.warm }

    pub fn better_request(&self) -> ShapeRequest {
        ShapeRequest {
            core: ShapeRequestCore {
                region: self.core.region.best_region(self.core.track.track()),
                track: self.core.track.clone(),
            },
            warm: self.warm
        }
    }
}
