use std::{collections::{HashMap}, sync::{Arc, Mutex}};

use crate::{ AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataStore, AllotmentRequest, Pitch};
use peregrine_toolkit::lock;

use super::{dustbinallotment::DustbinAllotmentRequest, lineargroup::{LinearRequestGroupName}, offsetallotment::OffsetAllotmentRequestCreator};
use super::lineargroup::LinearRequestGroup;

struct UniverseData {
    dustbin: Arc<DustbinAllotmentRequest>,
    requests: HashMap<LinearRequestGroupName,LinearRequestGroup<OffsetAllotmentRequestCreator>>
}

impl UniverseData {
    fn group(&self, name: &str) -> LinearRequestGroupName {
        if name.starts_with("window:") {
            LinearRequestGroupName::Screen(if name.ends_with("-over") { 1 } else { 0 })
        } else {
            LinearRequestGroupName::Track
        }
    }

    fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if name == "" {
            Some(AllotmentRequest::upcast(self.dustbin.clone()))
        } else {
            let metadata = allotment_metadata.get(name);
            if metadata.is_none() { return None; }
            let group = self.group(name);
            self.requests.entry(group.clone()).or_insert_with(|| LinearRequestGroup::new(&group,OffsetAllotmentRequestCreator())).make_request(allotment_metadata,name)
        }
    }

    fn union(&mut self, other: &UniverseData) {
        for (group_type,other_group) in other.requests.iter() {
            let self_group = self.requests.entry(group_type.clone()).or_insert_with(|| LinearRequestGroup::new(group_type,OffsetAllotmentRequestCreator()));
            self_group.union(other_group);
        }
    }

    fn get_all_metadata(&self,allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,group) in self.requests.iter() {
            group.get_all_metadata(allotment_metadata,out);
        }
    }

    fn allot(&mut self) {
        for (_,group) in self.requests.iter_mut() {
            group.allot();
        }
    }

    pub fn apply_pitch(&self, pitch: &mut Pitch) {
        if let Some(group) = self.requests.get(&LinearRequestGroupName::Track) {
            group.apply_pitch(pitch);
        }
    }
}

#[derive(Clone)]
pub struct Universe {
    data: Arc<Mutex<UniverseData>>,
    allotment_metadata: AllotmentMetadataStore
}

impl Universe {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> Universe {
        Universe {
            data: Arc::new(Mutex::new(UniverseData {
                requests: HashMap::new(),
                dustbin: Arc::new(DustbinAllotmentRequest())
            })),
            allotment_metadata: allotment_metadata.clone()
        }
    }

    pub fn make_metadata_report(&self) -> AllotmentMetadataReport {
        let mut metadata = vec![];
        lock!(self.data).get_all_metadata(&self.allotment_metadata, &mut metadata);
        AllotmentMetadataReport::new(metadata)
    }

    pub fn make_request(&self, name: &str) -> Option<AllotmentRequest> {
        lock!(self.data).make_request(&self.allotment_metadata,name)
    }

    pub fn union(&mut self, other: &Universe) {
        if Arc::ptr_eq(&self.data,&other.data) { return; }
        let mut self_data = lock!(self.data);
        let other_data = lock!(other.data);
        self_data.union(&other_data);
    }

    pub fn allot(&self) {
        lock!(self.data).allot();
    }

    pub fn apply_pitch(&self,pitch: &mut Pitch) {
        lock!(self.data).apply_pitch(pitch);
    }
}