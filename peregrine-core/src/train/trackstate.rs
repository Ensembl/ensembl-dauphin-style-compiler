use std::collections::HashSet;
use std::sync::{ Arc, Mutex };
use crate::core::track::Track;
use std::fmt::{ self, Display, Formatter };

#[derive(Clone,PartialEq)]
pub struct TrackStateSnapshot(HashSet<Track>);

impl TrackStateSnapshot {
    pub fn iter(&self) -> impl Iterator<Item=&Track> {
        self.0.iter()
    }
}

impl Display for TrackStateSnapshot {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut tracks : Vec<_> = self.0.iter().map(|x| x.to_string()).collect();
        tracks.sort();
        let tracks : String = tracks.join(", ");
        write!(f,"[{}]",tracks)
    }
}

struct TrackStateData {
    tracks_on: HashSet<Track>
}

impl TrackStateData {
    fn new() -> TrackStateData {
        TrackStateData {
            tracks_on: HashSet::new()
        }
    }

    fn track_on(&mut self, track: &Track, yn: bool) {
        if yn {
            self.tracks_on.insert(track.clone());
        } else {
            self.tracks_on.remove(track);
        }
    }

    fn snapshot(&mut self) -> TrackStateSnapshot {
        TrackStateSnapshot(self.tracks_on.iter().cloned().collect())
    }   
}

#[derive(Clone)]
pub struct TrackState(Arc<Mutex<TrackStateData>>);

impl TrackState {
    pub fn new() -> TrackState {
        TrackState(Arc::new(Mutex::new(TrackStateData::new())))
    }

    pub fn track_on(&self, track: &Track, yn: bool) {
        self.0.lock().unwrap().track_on(track,yn);
    }

    fn snapshot(&self) -> TrackStateSnapshot {
        self.0.lock().unwrap().snapshot()
    }
}
