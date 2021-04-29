use std::fmt::{ self, Display, Formatter };
use super::stick::StickId;
use crate::switch::trackconfig::TrackConfigList;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct Layout {
    stick: Option<StickId>,
    tracks: Option<TrackConfigList>
}

impl Layout {
    pub fn new(stick: &StickId, tcl: &TrackConfigList) -> Layout {
        Layout {
            tracks: Some(tcl.clone()),
            stick: Some(stick.clone())
        }
    }

    // XXX why?
    pub fn empty() -> Layout {
        Layout {
            tracks: None,
            stick: None
        }
    }

    pub fn ready(&self) -> bool { self.stick.is_some() }

    pub fn track_config_list(&self) -> &Option<TrackConfigList> { &self.tracks }
    pub fn stick(&self) -> &Option<StickId> { &self.stick }

    pub fn set_stick(&self, stick: &StickId) -> Layout {
        let mut out = self.clone();
        out.stick = Some(stick.clone());
        out
    }

    pub fn set_track_config_list(&self, track_config_list: &TrackConfigList) -> Layout {
        let mut out = self.clone();
        out.tracks = Some(track_config_list.clone());
        out
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tracks : Vec<_> = self.tracks.iter().map(|x| format!("{:?}",x)).collect();
        tracks.sort();
        if let Some(stick) = &self.stick {
            write!(f,"Layout(tracks={:?} stick={})",tracks,stick)
        } else {
            write!(f,"Layout(tracks={:?} *no stick*)",tracks)
        }
    }
}
