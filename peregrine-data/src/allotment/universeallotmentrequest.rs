use crate::{AllotmentMetadataStore, AllotmentRequest};

#[derive(Clone)]
pub struct UniverseAllotmentRequest {
    allotment_metadata: AllotmentMetadataStore
}

impl UniverseAllotmentRequest {
    pub fn new(allotment_metadata: &AllotmentMetadataStore) -> UniverseAllotmentRequest {
        UniverseAllotmentRequest {
            allotment_metadata: allotment_metadata.clone()
        }
    }

    pub fn get(&self, name: &str) -> Option<AllotmentRequest> {
        AllotmentRequest::make(&self.allotment_metadata,name)
    }
}