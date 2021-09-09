use std::fmt::{ self, Display, Formatter };
use super::stick::StickId;
use crate::switch::trackconfiglist::TrackConfigList;

#[derive(Clone,Hash,PartialEq,Eq)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Layout {
    stick: StickId,
    tracks: TrackConfigList,
    size: u64
}

impl Layout {
    pub fn new(stick: &StickId, size: u64, tcl: &TrackConfigList) -> Layout {
        Layout {
            tracks: tcl.clone(),
            stick: stick.clone(),
            size
        }
    }

    pub fn track_config_list(&self) -> &TrackConfigList { &self.tracks }
    pub fn stick(&self) -> &StickId { &self.stick }
    pub fn size(&self) -> u64 { self.size }

    pub fn set_stick(&mut self, stick: &StickId) {
        self.stick = stick.clone();
    }

    pub fn set_track_config_list(&mut self, track_config_list: &TrackConfigList) {
        self.tracks = track_config_list.clone();
    }
}
