use crate::lock;
use std::sync::{ Arc, Mutex };
use varea::VareaStore;
use crate::request::Channel;
use super::programregion::ProgramRegion;

pub struct LaneProgramStoreData {
    store: VareaStore<(Channel,String,ProgramRegion)>
}

impl LaneProgramStoreData {
    fn new() -> LaneProgramStoreData {
        LaneProgramStoreData {
            store: VareaStore::new()
        }
    }

    fn add(&mut self, lane_slice_range: &ProgramRegion, channel: &Channel, name: &str) {
        let varea_item = lane_slice_range.to_varea_item();
        self.store.add(varea_item,(channel.clone(),name.to_string(),lane_slice_range.clone()));
    }

    fn get(&self, lane_slice_range: &ProgramRegion) -> Option<(Channel,String,ProgramRegion)> {
        let varea_item = lane_slice_range.to_varea_item();
        let varea_search_term = self.store.search_item(&varea_item);
        let mut varea_matches = self.store.lookup(varea_search_term);
        varea_matches.next().map(|(c,n,ppr)| (c.clone(),n.to_string(),ppr.clone()))
    }
}

#[derive(Clone)]
pub struct LaneProgramStore(Arc<Mutex<LaneProgramStoreData>>);

impl LaneProgramStore {
    pub fn new() -> LaneProgramStore {
        LaneProgramStore(Arc::new(Mutex::new(LaneProgramStoreData::new())))
    }

    pub fn add(&self, lane_slice_range: &ProgramRegion, channel: &Channel, name: &str) {
        lock!(self.0).add(lane_slice_range,channel,name);
    }

    pub fn get(&self, lane_slice_range: &ProgramRegion) -> Option<(Channel,String,ProgramRegion)> {
        lock!(self.0).get(lane_slice_range)
    }
}
