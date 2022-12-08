use std::hash::Hash;
use crate::core::pixelsize::PixelSize;
use crate::core::{ Scale, StickId };
use crate::switch::trackconfig::TrackConfig;
use crate::switch::track::Track;
use serde::Serialize;
use serde::ser::SerializeSeq;

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash)]
pub struct Region {
    stick: StickId,
    scale: Scale,
    index: u64
}

impl Region {
    pub fn new(stick: &StickId, index: u64, scale: &Scale) -> Region {
        Region { stick: stick.clone(), scale: scale.clone(), index }
    }

    pub fn to_invariant(&self) -> Region {
        let mut out = self.clone();
        out.index = 0;
        out.scale = Scale::new(0);
        out
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

impl Serialize for Region {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.stick)?;
        seq.serialize_element(&self.scale)?;
        seq.serialize_element(&self.index)?;
        seq.end()
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct ShapeRequestCore {
    region: Region,
    track: TrackConfig,
    pixel_size: PixelSize
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
    pub fn new(region: &Region, track: &TrackConfig, pixel_size: &PixelSize, warm: bool) -> ShapeRequest {
        ShapeRequest {
            core: ShapeRequestCore {
                region: region.clone(),
                track: track.clone(),
                pixel_size: pixel_size.clone()
            },
            warm
        }
    }

    pub fn region(&self) -> &Region { &self.core.region }
    pub fn track(&self) -> &TrackConfig { &self.core.track }
    pub fn pixel_size(&self) -> &PixelSize { &self.core.pixel_size }
    pub fn warm(&self) -> bool { self.warm }

    pub fn better_request(&self) -> ShapeRequest {
        ShapeRequest {
            core: ShapeRequestCore {
                region: self.core.region.best_region(self.core.track.track()),
                track: self.core.track.clone(),
                pixel_size: self.core.pixel_size.clone()
            },
            warm: self.warm
        }
    }
}
