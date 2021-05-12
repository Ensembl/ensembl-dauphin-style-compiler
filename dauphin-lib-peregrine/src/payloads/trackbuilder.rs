use peregrine_data::{ lock, AllotmentHandle, ProgramRegionBuilder, ProgramName, Channel, Track, Switches, ProgramRegion };
use anyhow::{ anyhow as err };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

pub(crate) struct TrackBuilder {
    track: Track,
    mounts: Vec<(Vec<String>,bool)>
}

impl TrackBuilder {
    fn new(channel: &Channel, program: &str, min_scale: u64, max_scale: u64, scale_jump: u64) -> TrackBuilder {
        let program_name = ProgramName(channel.clone(),program.to_string());
        let track = Track::new(&program_name,min_scale,max_scale,scale_jump);
        TrackBuilder {
            track,
            mounts: vec![]
        }
    }

    pub(crate) fn add_tag(&mut self, tag: &str) { self.track.add_tag(tag); }
    pub(crate) fn add_allotment_request(&mut self, allotment: AllotmentHandle) {
        self.track.add_allotment_request(allotment);
    }

    pub(crate) fn track(&self) -> &Track { &self.track }

    pub(crate) fn add_mount(&mut self, path: &[&str], trigger: bool) {
        self.mounts.push((path.iter().map(|x| x.to_string()).collect(),trigger));
    }

    pub(crate) fn build(&mut self, switches: &Switches) -> ProgramRegion {
        let mut prb = ProgramRegionBuilder::new();
        for (path,trigger) in &self.mounts {
            prb.add_mount(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>(),*trigger);
        }
        prb.build(&self.track,switches)
    }
}

struct AllTracksBuilderData {
    next_id: usize,
    tracks: HashMap<usize,Arc<Mutex<TrackBuilder>>>
}

impl AllTracksBuilderData {
    fn new() -> AllTracksBuilderData {
        AllTracksBuilderData {
            next_id: 0,
            tracks: HashMap::new(),
        }
    }

    fn allocate(&mut self, channel: &Channel, program: &str, min_scale: u64, max_scale: u64, scale_jump: u64) -> usize {
        let id = self.next_id;
        let track_builder = TrackBuilder::new(channel,program,min_scale,max_scale,scale_jump);
        self.tracks.insert(id,Arc::new(Mutex::new(track_builder)));
        self.next_id += 1;
        id
    }

    fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<TrackBuilder>>> {
        Ok(self.tracks.get(&id).ok_or(err!("bad track id"))?.clone())
    }
}

#[derive(Clone)]
pub struct AllTracksBuilder(Arc<Mutex<AllTracksBuilderData>>);

impl AllTracksBuilder {
    pub fn new() -> AllTracksBuilder {
        AllTracksBuilder(Arc::new(Mutex::new(AllTracksBuilderData::new())))
    }

    pub fn allocate(&self, channel: &Channel, program: &str, min_scale: u64, max_scale: u64, scale_jump: u64) -> usize {
        lock!(self.0).allocate(channel,program,min_scale,max_scale,scale_jump)
    }

    pub(crate) fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<TrackBuilder>>> {
        lock!(self.0).get(id)
    }
}