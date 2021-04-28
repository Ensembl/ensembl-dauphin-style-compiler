use std::collections::{ BTreeSet };
use std::fmt::{ self, Display, Formatter };
use super::stick::StickId;
use super::track::Track;

#[derive(Clone,Debug,Hash,PartialEq,Eq)]
pub struct Layout {
    tracks: BTreeSet<Track>,
    stick: Option<StickId>
}

impl Layout {
    pub fn new(stick: &StickId) -> Layout {
        Layout {
            tracks: BTreeSet::new(),
            stick: Some(stick.clone())
        }
    }

    pub fn empty() -> Layout {
        Layout {
            tracks: BTreeSet::new(),
            stick: None
        }
    }

    pub fn ready(&self) -> bool { self.stick.is_some() }

    pub fn tracks(&self) -> &BTreeSet<Track> { &self.tracks }
    pub fn stick(&self) -> &Option<StickId> { &self.stick }

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
        out.stick = Some(stick.clone());
        out
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tracks : Vec<_> = self.tracks.iter().map(|x| x.to_string()).collect();
        tracks.sort();
        if let Some(stick) = &self.stick {
            write!(f,"Layout(tracks={} stick={})",tracks.join(", "),stick)
        } else {
            write!(f,"Layout(tracks={} *no stick*)",tracks.join(", "))
        }
    }
}
