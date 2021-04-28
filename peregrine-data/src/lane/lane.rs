use crate::core::track::Track;
use crate::core::{ Scale, StickId };
use crate::index::StickStore;
use super::laneprogramstore::LaneProgramStore;
use super::programregion::ProgramRegion;
use super::lanerunstore::LaneRun;
use crate::util::message::{ DataMessage };
use serde_cbor::Value as CborValue;

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Lane {
    stick: StickId,
    scale: Scale,
    track: Track,
    index: u64
}

impl Lane {
    pub fn new(stick: StickId, index: u64, scale: Scale, track: Track) -> Lane {
        Lane { stick, scale, track, index }
    }

    pub fn stick_id(&self) -> &StickId { &self.stick }
    pub fn track(&self) -> &Track { &self.track }
    pub fn index(&self) -> u64 { self.index }
    pub fn scale(&self) -> &Scale { &self.scale }

    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(CborValue::Array(vec![
            self.stick.serialize()?,self.scale.serialize()?,
            self.track.serialize()?,CborValue::Integer(self.index as i128)
        ]))
    }

    pub fn min_value(&self) -> u64 {
        self.scale.bp_in_carriage() * self.index
    }

    pub fn max_value(&self) -> u64 {
        self.scale.bp_in_carriage() * (self.index+1)
    }

    fn map_scale(&self, scale: &Scale) -> u64 {
        self.index >> (scale.get_index() - self.scale.get_index())
    }

    fn to_candidate(&self, ppr: &ProgramRegion) ->Lane {
        let scale = ppr.scale_up(&self.scale);
        let index = self.map_scale(&scale);
        Lane {
            stick: self.stick.clone(),
            scale, index,
            track: self.track.clone()
        }
    }

    pub async fn build_lane_run(&self, stick_store: &StickStore, lane_program_store: &LaneProgramStore) -> Result<LaneRun,DataMessage> {
        let tags : Vec<String> = stick_store.get(&self.stick).await?.as_ref().tags().iter().cloned().collect();
        let mut ppr = ProgramRegion::new();
        ppr.set_stick_tags(&tags);
        ppr.set_scale(self.scale.clone(),self.scale.next_scale());
        ppr.set_tracks(&[self.track.clone()]);
        let (channel,prog,ppr) = lane_program_store.get(&ppr)
            .ok_or_else(|| DataMessage::NoLaneProgram(self.clone()))?;
        let candidate = self.to_candidate(&ppr);
        Ok(LaneRun::new(channel,&prog,&candidate))
    }
}
