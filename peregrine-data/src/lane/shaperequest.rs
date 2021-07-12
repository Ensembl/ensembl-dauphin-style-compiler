use crate::core::{ Scale, StickId };
use crate::util::message::{ DataMessage };
use serde_cbor::Value as CborValue;
use crate::switch::trackconfig::TrackConfig;
use crate::switch::track::Track;

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

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![
            self.stick.serialize()?,self.scale.serialize()?,CborValue::Integer(self.index as i128)
        ]))
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

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
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
