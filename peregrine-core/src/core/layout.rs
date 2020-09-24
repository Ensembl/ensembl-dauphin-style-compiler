use std::collections::HashSet;
use std::fmt::{ self, Display, Formatter };
use super::focus::Focus;
use super::stick::StickId;
use super::track::Track;

#[derive(Clone,PartialEq)]
pub struct Layout {
    tracks: HashSet<Track>,
    focus: Focus,
    stick: StickId
}

impl Layout {
    pub fn new(stick: &StickId) -> Layout {
        Layout {
            tracks: HashSet::new(),
            focus: Focus::new(None),
            stick: stick.clone()
        }
    }

    pub fn tracks(&self) -> &HashSet<Track> { &self.tracks }
    pub fn focus(&self) -> &Focus { &self.focus }
    pub fn stick(&self) -> &StickId { &self.stick }

    pub fn track_on(&self, track: &Track, yn: bool) -> Layout {
        let mut out = self.clone();
        if yn {
            out.tracks.insert(track.clone());
        } else {
            out.tracks.remove(track);
        }
        out
    }

    pub fn set_stick(&self, stick: &StickId) -> Layout {
        let mut out = self.clone();
        out.stick = stick.clone();
        out
    }

    pub fn set_focus(&self, focus: &Focus) -> Layout {
        let mut out = self.clone();
        out.focus = focus.clone();
        out
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tracks : Vec<_> = self.tracks.iter().map(|x| x.to_string()).collect();
        tracks.sort();
        write!(f,"Layout(tracks={} focus={} stick={})",tracks.join(", "),self.focus,self.stick)
    }
}
