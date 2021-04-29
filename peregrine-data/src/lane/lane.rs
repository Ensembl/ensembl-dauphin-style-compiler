use crate::core::{ Scale, StickId };
use crate::index::StickStore;
use crate::agent::laneprogramlookup::LaneProgramLookup;
use super::programregion::{ ProgramRegion, ProgramRegionQuery };
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

    fn map_scale(&self, scale: &Scale) -> u64 {
        self.index >> (scale.get_index() - self.scale.get_index())
    }

    fn to_candidate(&self, program_region: &ProgramRegion) -> Lane {
        let scale = program_region.scale_up(&self.scale);
        let index = self.map_scale(&scale);
        Lane {
            stick: self.stick.clone(),
            scale, index,
            track: self.track.clone()
        }
    }

    pub async fn scaler(&self, stick_store: &StickStore, lane_program_lookup: &LaneProgramLookup) -> Result<Lane,DataMessage> {
        let tags : Vec<String> = stick_store.get(&self.stick).await?.as_ref().tags().iter().cloned().collect();
        let program_region_query = ProgramRegionQuery::new(&tags,&self.scale,self.track.track().program_name());
        let (_program_name,program_region) = lane_program_lookup.get(&program_region_query)
            .ok_or_else(|| DataMessage::NoLaneProgram(self.clone()))?;
        let candidate = self.to_candidate(&program_region);
        Ok(candidate)
    }
}
