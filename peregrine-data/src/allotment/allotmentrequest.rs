use std::hash::Hash;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;

use crate::{AllotmentGroup, AllotmentMetadata, AllotmentMetadataStore, AllotmentPosition };

#[cfg_attr(debug_assertions,derive(Debug))]
struct AllotmentRequestData {
    metadata: AllotmentMetadata
}

impl Hash for AllotmentRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        lock!(self.0).metadata.hash(state);
    }
}

impl PartialEq for AllotmentRequest {
    fn eq(&self, other: &Self) -> bool {
        let a = lock!(self.0).metadata.clone();
        let b = lock!(other.0).metadata.clone();
        a == b
    }
}

impl Eq for AllotmentRequest {}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct AllotmentRequest(Arc<Mutex<AllotmentRequestData>>);

impl AllotmentRequest {
    pub fn new(allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        allotment_metadata.get(name).map(|metadata| {
            AllotmentRequest(Arc::new(Mutex::new(AllotmentRequestData {
                metadata
            })))
        })
    }

    pub fn name(&self) -> String { lock!(self.0).metadata.name().to_string() }
    pub fn allotment_group(&self) -> AllotmentGroup { lock!(self.0).metadata.allotment_group() }
    pub fn is_dustbin(&self) -> bool { lock!(self.0).metadata.is_dustbin() }
    pub fn priority(&self) -> i64 { lock!(self.0).metadata.priority() }
    pub fn metadata(&self) -> AllotmentMetadata { lock!(self.0).metadata.clone() }

    pub fn update_metadata(&self, position: &AllotmentPosition) -> AllotmentRequest {
        let metadata = lock!(self.0).metadata.clone().update_metadata(position);
        AllotmentRequest(Arc::new(Mutex::new(AllotmentRequestData {
            metadata
        })))
    }
}
