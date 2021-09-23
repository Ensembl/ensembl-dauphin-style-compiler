use std::{collections::HashSet, sync::{Arc, Mutex}};

use crate::{AllotmentMetadata, AllotmentMetadataReport, AllotmentMetadataRequest, AllotmentMetadataStore, AllotmentPosition, AllotmentRequest, OffsetSize};
use peregrine_toolkit::lock;

struct UniverseAllotmentRequestData {
    used_names: HashSet<String>
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
                used_names: HashSet::new()
            })),
            allotment_metadata: allotment_metadata.clone()
        }
    }

    pub fn make_metadata_report(&self) -> AllotmentMetadataReport {
        let mut metadata = vec![];
        for name in &lock!(self.data).used_names {
            if let Some(this_metadata) = self.allotment_metadata.get(name) {
                let mut full_metadata = AllotmentMetadataRequest::rebuild(&this_metadata);
                /* XXX */
                full_metadata.add_pair("type","track");
                full_metadata.add_pair("offset","-1");
                full_metadata.add_pair("height","-1");
                metadata.push(AllotmentMetadata::new(full_metadata));
            }
        }
        AllotmentMetadataReport::new(metadata)
    }

    pub fn make_request(&self, name: &str) -> Option<AllotmentRequest> {
        lock!(self.data).used_names.insert(name.to_string());
        AllotmentRequest::make(&self.allotment_metadata,name)
    }

    pub fn union(&mut self, other: &UniverseAllotmentRequest) {
        for name in &lock!(other.data).used_names {
            self.make_request(name);
        }
    }

    pub fn allot(&mut self) {
        // XXX
    }
}