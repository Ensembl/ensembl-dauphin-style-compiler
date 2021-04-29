use peregrine_data::{ lock, ProgramRegion, ProgramRegionBuilder };
use anyhow::{ anyhow as err };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };

struct LaneBuilderData {
    next_id: usize,
    lanes: HashMap<usize,Arc<Mutex<ProgramRegionBuilder>>>
}

impl LaneBuilderData {
    fn new() -> LaneBuilderData {
        LaneBuilderData {
            next_id: 0,
            lanes: HashMap::new()
        }
    }

    fn allocate(&mut self) -> usize {
        let psr = Arc::new(Mutex::new(ProgramRegionBuilder::new()));
        let id = self.next_id;
        self.lanes.insert(id,psr.clone());
        self.next_id += 1;
        id
    }

    fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<ProgramRegionBuilder>>> {
        Ok(self.lanes.get(&id).ok_or(err!("bad lane id"))?.clone())
    }
}

#[derive(Clone)]
pub struct LaneBuilder(Arc<Mutex<LaneBuilderData>>);

impl LaneBuilder {
    pub fn new() -> LaneBuilder {
        LaneBuilder(Arc::new(Mutex::new(LaneBuilderData::new())))
    }

    pub fn allocate(&self) -> usize {
        lock!(self.0).allocate()
    }

    pub fn get(&self, id: usize) -> anyhow::Result<Arc<Mutex<ProgramRegionBuilder>>> {
        lock!(self.0).get(id)
    }
}