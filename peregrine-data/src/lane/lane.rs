use crate::core::{ Scale, StickId };
use crate::util::message::{ DataMessage };
use serde_cbor::Value as CborValue;
use crate::switch::trackconfig::TrackConfig;

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Region {
    stick: StickId,
    scale: Scale,
    index: u64
}

impl Region {
    pub fn new(stick: StickId, index: u64, scale: Scale) -> Region {
        Region { stick, scale, index }
    }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![
            self.stick.serialize()?,self.scale.serialize()?,CborValue::Integer(self.index as i128)
        ]))
    }
}

#[derive(Clone,Debug,Eq,Hash,PartialEq)] // XXX all needed now?
pub struct Lane {
    stick: StickId,
    scale: Scale,
    track: TrackConfig,
    index: u64
}

impl Lane {
    pub fn new(stick: StickId, index: u64, scale: Scale, track: TrackConfig) -> Lane {
        Lane { stick, scale, track, index }
    }

    pub fn stick_id(&self) -> &StickId { &self.stick }
    pub fn track_config(&self) -> &TrackConfig { &self.track }
    pub fn index(&self) -> u64 { self.index }
    pub fn scale(&self) -> &Scale { &self.scale }

    pub fn min_value(&self) -> u64 {
        self.scale.bp_in_carriage() * self.index
    }

    pub fn max_value(&self) -> u64 {
        self.scale.bp_in_carriage() * (self.index+1)
    }
}
