use std::{hash::Hash, sync::Mutex};
use std::sync::Arc;
use peregrine_toolkit::lock;

use super::basicallotmentspec::BasicAllotmentSpec;
use crate::{Allotment, DataMessage, AllotmentMetadata, AllotmentMetadataRequest};

use super::allotment::AllotmentImpl;
use super::{allotment::CoordinateSystem, dustbinallotment::DustbinAllotment};

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

pub trait AgnosticAllotmentRequestImpl {
    fn name(&self) -> &str;
    fn is_dustbin(&self) -> bool;
    fn priority(&self) -> i64;
    fn allotment(&self) -> Result<Allotment,DataMessage>;
    fn up(self: Arc<Self>) -> Arc<dyn AgnosticAllotmentRequestImpl>;
    fn register_usage(&self, max: i64);
    fn coord_system(&self) -> CoordinateSystem;
    fn depth(&self) -> i8;
}

#[derive(Clone)]
pub struct AllotmentRequest(Arc<dyn AgnosticAllotmentRequestImpl>);

impl AllotmentRequest {
    pub(crate) fn upcast<T>(request: Arc<T>) -> AllotmentRequest where T: AgnosticAllotmentRequestImpl + 'static + ?Sized {
        AllotmentRequest(request.up())
    }

    pub fn name(&self) -> String { self.0.name().to_string() }
    pub fn is_dustbin(&self) -> bool { self.0.is_dustbin() }
    pub fn priority(&self) -> i64 { self.0.priority() }
    pub fn depth(&self) -> i8 { self.0.depth() }
    pub fn allotment(&self) -> Result<Allotment,DataMessage> { self.0.allotment() }
    pub fn coord_system(&self) -> CoordinateSystem { self.0.coord_system() }
    pub fn register_usage(&self, max: i64) { self.0.register_usage(max); }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}

pub struct AllotmentRequestImpl<T> {
    metadata: AllotmentMetadata,
    name: String,
    priority: i64,
    allotment: Mutex<Option<Arc<T>>>,
    coord_system: CoordinateSystem,
    depth: i8,
    max: Mutex<i64>
}

impl<T> AllotmentRequestImpl<T> {
    pub fn new(metadata: &AllotmentMetadata, coord_system: &CoordinateSystem, depth: i8) -> AllotmentRequestImpl<T> {
        AllotmentRequestImpl {
            name: BasicAllotmentSpec::from_spec(metadata.name()).name().to_string(),
            priority: metadata.priority(),
            metadata: metadata.clone(),
            allotment: Mutex::new(None),
            depth,
            coord_system: coord_system.clone(),
            max: Mutex::new(0)
        }
    }

    pub fn set_allotment(&self, value: Arc<T>) {
        if &self.name != "" {
            *self.allotment.lock().unwrap() = Some(value);
        }
    }

    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }
    pub fn max_used(&self) -> i64 { *self.max.lock().unwrap() }

    pub fn base_allotment(&self) -> Option<Arc<T>> {
        self.allotment.lock().unwrap().as_ref().cloned()
    }
}

impl<T: AllotmentImpl + 'static> AgnosticAllotmentRequestImpl for AllotmentRequestImpl<T> {
    fn name(&self) -> &str { &self.name }
    fn is_dustbin(&self) -> bool { &self.name == "" }
    fn priority(&self) -> i64 { self.priority }
    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        match self.allotment.lock().unwrap().clone() {
            Some(imp) => Ok(Allotment::new(imp)),
            None => Err(DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name())))
        }
    }

    fn register_usage(&self, max: i64) {
        let mut self_max = lock!(self.max);
        *self_max = (*self_max).max(max)
    }

    fn up(self: Arc<Self>) -> Arc<dyn AgnosticAllotmentRequestImpl> { self }
}

impl AllotmentRequestImpl<DustbinAllotment> {
    pub fn new_dustbin() -> AllotmentRequestImpl<DustbinAllotment> {
        AllotmentRequestImpl {
            name: String::new(),
            priority: 0,
            metadata: AllotmentMetadata::new(AllotmentMetadataRequest::dustbin()),
            allotment: Mutex::new(None),
            depth: 0,
            coord_system: CoordinateSystem::Window,
            max: Mutex::new(0)
        }
    }
}
