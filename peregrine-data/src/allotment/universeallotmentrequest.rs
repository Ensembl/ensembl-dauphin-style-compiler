use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}};

use crate::{AllotmentGroup, AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentPosition, AllotmentRequest, OffsetSize};
use peregrine_toolkit::lock;

struct AllotmentRequestGroupStore {
    requests: HashMap<String,AllotmentRequest>
}

impl AllotmentRequestGroupStore {
    fn new() -> AllotmentRequestGroupStore {
        AllotmentRequestGroupStore {
            requests: HashMap::new()
        }
    }

    pub fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        if !self.requests.contains_key(name) {
            let request = AllotmentRequest::make(allotment_metadata,name);
            if request.is_none() { return None; }
            self.requests.insert(name.to_string(),request.unwrap());
        }
        Some(self.requests.get(name).unwrap().clone())
    }

    fn union(&mut self, other: &AllotmentRequestGroupStore) {
        for (name,request) in other.requests.iter() {
            if !self.requests.contains_key(name) {
                self.requests.insert(name.to_string(),request.clone());
            }
        }
    }

    fn get_all_metadata(&self, allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,request) in self.requests.iter() {
            if let Some(this_metadata) = allotment_metadata.get(&request.name()) {
                let mut full_metadata = AllotmentMetadataRequest::rebuild(&this_metadata);
                /* XXX */
                full_metadata.add_pair("type","track");
                full_metadata.add_pair("offset","-1");
                full_metadata.add_pair("height","-1");
                out.push(AllotmentMetadata::new(full_metadata));
            }
        }
    }
}

struct UniverseAllotmentRequestData {
    requests: HashMap<AllotmentGroup,AllotmentRequestGroupStore>
}

impl UniverseAllotmentRequestData {
    fn make_request(&mut self, allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        let xxx = AllotmentRequest::make(allotment_metadata,name); // XXX
        if xxx.is_none() { return None; }
        let group = xxx.unwrap().allotment_group();
        self.requests.entry(group).or_insert_with(|| AllotmentRequestGroupStore::new()).make_request(allotment_metadata,name)
    }

    fn union(&mut self, other: &UniverseAllotmentRequestData) {
        for (group_type,other_group) in other.requests.iter() {
            let self_group = self.requests.entry(group_type.clone()).or_insert_with(|| AllotmentRequestGroupStore::new());
            self_group.union(other_group);
        }
    }

    fn get_all_metadata(&self,allotment_metadata: &AllotmentMetadataStore, out: &mut Vec<AllotmentMetadata>) {
        for (_,group) in self.requests.iter() {
            group.get_all_metadata(allotment_metadata,out);
        }
    }
}

#[derive(Clone)]
pub struct UniverseAllotmentRequest {
    data: Arc<Mutex<UniverseAllotmentRequestData>>,
    allotment_metadata: AllotmentMetadataStore
}

impl UniverseAllotmentRequest {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> UniverseAllotmentRequest {
        UniverseAllotmentRequest {
            data: Arc::new(Mutex::new(UniverseAllotmentRequestData {
                requests: HashMap::new()
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

    pub fn union(&mut self, other: &UniverseAllotmentRequest) {
        if Arc::ptr_eq(&self.data,&other.data) { return; }
        let mut self_data = lock!(self.data);
        let other_data = lock!(other.data);
        self_data.union(&other_data);
    }

    pub fn allot(&mut self) {
        // XXX
    }
}