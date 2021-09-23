use std::collections::HashSet;

use crate::{AllotmentMetadataStore, AllotmentRequest, AllotterMetadata};

#[derive(Clone)]
pub struct UniverseAllotmentRequest {
    used_names: HashSet<String>,
    allotment_metadata: AllotmentMetadataStore
}

impl UniverseAllotmentRequest {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> UniverseAllotmentRequest {
        UniverseAllotmentRequest {
            used_names: HashSet::new(),
            allotment_metadata: allotment_metadata.clone()
        }
    }

    pub fn make_request(&self, name: &str) -> Option<AllotmentRequest> {
        AllotmentRequest::make(&self.allotment_metadata,name)
    }
}