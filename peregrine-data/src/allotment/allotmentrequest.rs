use std::hash::Hash;
use std::sync::{Arc, Mutex};
use peregrine_toolkit::refs::Upcast;
use as_dyn_trait::as_dyn_trait;

use crate::{Allotment, AllotmentDirection, AllotmentGroup, AllotmentMetadata, DataMessage};

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

#[as_dyn_trait]
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
    pub(super) fn upcast<T>(request: Arc<T>) -> AllotmentRequest where T: AllotmentRequestImpl + 'static + ?Sized {
        AllotmentRequest(request.as_dyn_allotment_request_impl())
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

pub struct BaseAllotmentRequest<T> {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<T>>>,
    group: AllotmentGroup
}

impl<T> BaseAllotmentRequest<T> {
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

impl<T: AllotmentImpl + 'static> AllotmentRequestImpl for BaseAllotmentRequest<T> {
    fn name(&self) -> String { self.metadata.name().to_string() }
    fn allotment_group(&self) -> AllotmentGroup { self.group.clone() }
    fn is_dustbin(&self) -> bool { false }
    fn priority(&self) -> i64 { self.metadata.priority() }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        let allotment = self.allotment.lock().unwrap().clone();
        if allotment.is_none() { return Err(DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name()))); }
        let allotment = allotment.unwrap();
        Ok(Allotment::new(allotment))
    }
}
