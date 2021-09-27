use std::hash::Hash;
use std::sync::{Arc};

use crate::{Allotment, AllotmentGroup, AllotmentMetadata, AllotmentMetadataStore, DataMessage};

use super::allotment::AllotmentImpl;

impl Hash for AllotmentRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name().hash(state);
    }
}

impl PartialEq for AllotmentRequest {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.name().clone();
        let b = other.0.name().clone();
        a == b
    }
}

impl Eq for AllotmentRequest {}

pub trait AsAllotmentRequestImpl {
    fn as_allotment_request_impl<'a>(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl + 'a> where Self: 'a;
}

impl<T: AllotmentRequestImpl + Sized> AsAllotmentRequestImpl for T {
    fn as_allotment_request_impl<'a>(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl + 'a> where Self: 'a { self }
}

pub trait AllotmentRequestImpl {
    fn name(&self) -> String;
    fn allotment_group(&self) -> AllotmentGroup;
    fn is_dustbin(&self) -> bool;
    fn priority(&self) -> i64;
    fn allotment(&self) -> Result<Allotment,DataMessage>;
}

#[derive(Clone)]
pub struct AllotmentRequest(Arc<dyn AllotmentRequestImpl>);

impl AllotmentRequest {
    pub(super) fn upcast<T>(request: Arc<T>) -> AllotmentRequest where T: AsAllotmentRequestImpl + 'static {
        AllotmentRequest(request.as_allotment_request_impl())
    }

    pub fn name(&self) -> String { self.0.name().to_string() }
    pub fn allotment_group(&self) -> AllotmentGroup { self.0.allotment_group() }
    pub fn is_dustbin(&self) -> bool { self.0.is_dustbin() }
    pub fn priority(&self) -> i64 { self.0.priority() }
    pub fn allotment(&self) -> Result<Allotment,DataMessage> { self.0.allotment() }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}
