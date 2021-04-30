use std::fmt::{ self, Display, Formatter };
use super::stick::StickId;
use crate::switch::trackconfiglist::TrackConfigList;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct Layout {
    stick: StickId,
    tracks: TrackConfigList
}

impl Layout {
    pub fn new(stick: &StickId, tcl: &TrackConfigList) -> Layout {
        Layout {
            tracks: tcl.clone(),
            stick: stick.clone()
        }
    }

    pub fn track_config_list(&self) -> &TrackConfigList { &self.tracks }
    pub fn stick(&self) -> &StickId { &self.stick }

    pub fn set_stick(&mut self, stick: &StickId) {
        self.stick = stick.clone();
    }

    pub fn set_track_config_list(&mut self, track_config_list: &TrackConfigList) {
        self.tracks = track_config_list.clone();
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"Layout(tracks={:?} stick={})",self.tracks,self.stick)
    }
}
