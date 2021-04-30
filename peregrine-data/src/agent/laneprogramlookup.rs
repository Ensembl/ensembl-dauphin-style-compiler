use crate::lock;
use std::sync::{ Arc, Mutex };
use varea::VareaStore;
use crate::request::Channel;
use crate::lane::programregion::{ ProgramRegion, ProgramRegionQuery };
use crate::lane::programname::ProgramName;

pub struct LaneProgramLookupData {
    store: VareaStore<(ProgramName,ProgramRegion)>
}

impl LaneProgramLookupData {
    fn new() -> LaneProgramLookupData {
        LaneProgramLookupData {
            store: VareaStore::new()
        }
    }

    fn add(&mut self, lane_slice_range: &ProgramRegion, program_name: &ProgramName) {
        let varea_item = lane_slice_range.to_varea_item();
        self.store.add(varea_item,(program_name.clone(),lane_slice_range.clone()));
    }

    fn get(&self, program_region_query: &ProgramRegionQuery) -> Option<ProgramRegion> {
        let varea_item = program_region_query.to_varea_item();
        let varea_search_term = self.store.search_item(&varea_item);
        let mut varea_matches = self.store.lookup(varea_search_term);
        varea_matches.next().map(|(program_name,ppr)| ppr.clone())
    }
}

#[derive(Clone)]
pub struct LaneProgramLookup(Arc<Mutex<LaneProgramLookupData>>);

impl LaneProgramLookup {
    pub fn new() -> LaneProgramLookup {
        LaneProgramLookup(Arc::new(Mutex::new(LaneProgramLookupData::new())))
    }

    pub fn add(&self, lane_slice_range: &ProgramRegion, program_name: &ProgramName) {
        lock!(self.0).add(lane_slice_range,program_name);
    }

    pub fn get(&self, program_region_query: &ProgramRegionQuery) -> Option<ProgramRegion> {
        lock!(self.0).get(program_region_query)
    }
}
