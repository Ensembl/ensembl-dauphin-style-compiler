use peregrine_data::{ lock, ProgramRegion, ProgramRegionBuilder, ProgramName, Channel, Track };
use anyhow::{ anyhow as err };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

pub(crate) struct TrackBuilder {
    pub track: Track,
    pub prb: ProgramRegionBuilder
}

struct LaneBuilderData {
    next_id: usize,
    lanes: HashMap<usize,Arc<Mutex<TrackBuilder>>>
}

impl LaneBuilderData {
    fn new() -> LaneBuilderData {
        LaneBuilderData {
            next_id: 0,
            lanes: HashMap::new()
        }
    }

    fn allocate(&mut self, channel: &Channel, program: &str, min_scale: u64, max_scale: u64, scale_jump: u64) -> usize {
        let id = self.next_id;
        let program_name = ProgramName(channel.clone(),program.to_string());
        let track = Track::new(&program_name,min_scale,max_scale,scale_jump);
        let track_builder = TrackBuilder {
            track,
            prb: ProgramRegionBuilder::new()
        };
        self.lanes.insert(id,Arc::new(Mutex::new(track_builder)));
        self.next_id += 1;
        id
    }

    fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<TrackBuilder>>> {
        Ok(self.lanes.get(&id).ok_or(err!("bad lane id"))?.clone())
    }
}

#[derive(Clone)]
pub struct LaneBuilder(Arc<Mutex<LaneBuilderData>>);

impl LaneBuilder {
    pub fn new() -> LaneBuilder {
        LaneBuilder(Arc::new(Mutex::new(LaneBuilderData::new())))
    }

    pub fn allocate(&self, channel: &Channel, program: &str, min_scale: u64, max_scale: u64, scale_jump: u64) -> usize {
        lock!(self.0).allocate(channel,program,min_scale,max_scale,scale_jump)
    }

    pub(crate) fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<TrackBuilder>>> {
        lock!(self.0).get(id)
    }
}