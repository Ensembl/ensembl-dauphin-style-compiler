use std::hash::Hash;
use std::sync::Arc;
use crate::{Allotment, AllotmentGroup, DataMessage};
use crate::shape::shape::FilterMinMax;

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

pub trait AllotmentRequestImpl {
    fn name(&self) -> String;
    fn allotment_group(&self) -> AllotmentGroup;
    fn is_dustbin(&self) -> bool;
    fn priority(&self) -> i64;
    fn allotment(&self) -> Result<Allotment,DataMessage>;
    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl>;
    fn register_usage(&self, max: i64);
    fn filter_min_max(&self) -> FilterMinMax;
}

#[derive(Clone)]
pub struct AllotmentRequest(Arc<dyn AllotmentRequestImpl>);

impl AllotmentRequest {
    pub(super) fn upcast<T>(request: Arc<T>) -> AllotmentRequest where T: AllotmentRequestImpl + 'static + ?Sized {
        AllotmentRequest(request.up())
    }

    pub fn name(&self) -> String { self.0.name().to_string() }
    pub fn allotment_group(&self) -> AllotmentGroup { self.0.allotment_group() }
    pub fn is_dustbin(&self) -> bool { self.0.is_dustbin() }
    pub fn priority(&self) -> i64 { self.0.priority() }
    pub fn allotment(&self) -> Result<Allotment,DataMessage> { self.0.allotment() }
    pub fn filter_min_max(&self) -> FilterMinMax { self.0.filter_min_max() }
    pub fn register_usage(&self, max: i64) { self.0.register_usage(max); }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}
