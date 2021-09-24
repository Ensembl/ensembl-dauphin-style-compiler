use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crate::{Allotment, AllotmentDirection, AllotmentGroup, AllotmentMetadata, DataMessage};

use super::allotment::AsAllotmentImpl;

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

pub struct BaseAllotmentRequest<T: AsAllotmentImpl> {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<T>>>,
    group: AllotmentGroup
}

impl<T: AsAllotmentImpl> BaseAllotmentRequest<T> {
    pub fn new(metadata: &AllotmentMetadata, group: &AllotmentGroup) -> BaseAllotmentRequest<T> {
        BaseAllotmentRequest { metadata: metadata.clone(), allotment: Mutex::new(None), group: group.clone() }
    }

    pub fn set_allotment(&self, value: Arc<T>) {
        *self.allotment.lock().unwrap() = Some(value);
    }

    pub fn direction(&self) -> AllotmentDirection { self.group.direction() }
    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }

    pub fn base_allotment(&self) -> Option<Arc<T>> {
        self.allotment.lock().unwrap().as_ref().cloned()
    }
}

impl<T: AsAllotmentImpl + 'static> AllotmentRequestImpl for BaseAllotmentRequest<T> {
    fn name(&self) -> String { self.metadata.name().to_string() }
    fn allotment_group(&self) -> AllotmentGroup { self.group.clone() }
    fn is_dustbin(&self) -> bool { false }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        Ok(Allotment::new(self.allotment.lock().unwrap().clone()
            .ok_or_else(|| DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name())))?))
    }
}
