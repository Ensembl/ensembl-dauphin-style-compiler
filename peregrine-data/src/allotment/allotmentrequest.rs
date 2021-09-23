use std::hash::Hash;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;

use crate::{AllotmentGroup, AllotmentMetadata, AllotmentMetadataStore };

#[cfg_attr(debug_assertions,derive(Debug))]
struct AllotmentRequestData {
    metadata: AllotmentMetadata
}

impl AllotmentRequestImpl for AllotmentRequestData {
    fn name(&self) -> String { self.metadata.name().to_string() }
    fn allotment_group(&self) -> AllotmentGroup { self.metadata.allotment_group() }
    fn is_dustbin(&self) -> bool { self.metadata.is_dustbin() }
    fn priority(&self) -> i64 { self.metadata.priority() }
}

impl Hash for AllotmentRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        lock!(self.0).name().hash(state);
    }
}

impl PartialEq for AllotmentRequest {
    fn eq(&self, other: &Self) -> bool {
        let a = lock!(self.0).name().clone();
        let b = lock!(other.0).name().clone();
        a == b
    }
}

impl Eq for AllotmentRequest {}

pub trait AllotmentRequestImpl {
    fn name(&self) -> String;
    fn allotment_group(&self) -> AllotmentGroup;
    fn is_dustbin(&self) -> bool;
    fn priority(&self) -> i64;
    //    fn make_allotment(&self) -> Box<dyn AllotmentImpl>;
}

#[derive(Clone)]
pub struct AllotmentRequest(Arc<Mutex<Box<dyn AllotmentRequestImpl>>>);

impl AllotmentRequest {
    pub(super) fn make(allotment_metadata: &AllotmentMetadataStore, name: &str) -> Option<AllotmentRequest> {
        allotment_metadata.get(name).map(|metadata| {
            AllotmentRequest(Arc::new(Mutex::new(Box::new(AllotmentRequestData {
                metadata
            }))))
        })
    }

    pub fn name(&self) -> String { lock!(self.0).name().to_string() }
    pub fn allotment_group(&self) -> AllotmentGroup { lock!(self.0).allotment_group() }
    pub fn is_dustbin(&self) -> bool { lock!(self.0).is_dustbin() }
    pub fn priority(&self) -> i64 { lock!(self.0).priority() }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}
