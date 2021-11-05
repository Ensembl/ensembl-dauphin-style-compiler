use crate::core::{ Scale, StickId };
use serde::Serializer;
use serde::ser::SerializeSeq;
use crate::switch::trackconfig::TrackConfig;
use crate::switch::track::Track;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Region {
    stick: StickId,
    scale: Scale,
    index: u64
}

impl serde::Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.stick)?;
        seq.serialize_element(&self.scale)?;
        seq.serialize_element(&self.index)?;
        seq.end()
    }
}

impl Region {
    pub fn new(stick: &StickId, index: u64, scale: &Scale) -> Region {
        Region { stick: stick.clone(), scale: scale.clone(), index }
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
pub struct ShapeRequest {
    region: Region,
    track: TrackConfig
}

impl ShapeRequest {
    pub fn new(region: &Region, track: &TrackConfig) -> ShapeRequest {
        ShapeRequest {
            region: region.clone(),
            track: track.clone()
        }
    }

    pub fn region(&self) -> &Region { &self.region }
    pub fn track(&self) -> &TrackConfig { &self.track }

    pub fn better_request(&self) -> ShapeRequest {
        ShapeRequest {
            region: self.region.best_region(self.track.track()),
            track: self.track.clone()
        }
    }
}
