use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use crate::{Allotment, AllotmentMetadata, DataMessage, AllotmentMetadataRequest, spacebase::spacebase::SpaceBasePoint, SpaceBasePointRef};
use super::{allotment::{AllotmentImpl, CoordinateSystem}, allotmentrequest::{AllotmentRequestImpl}, basicallotmentspec::BasicAllotmentSpec, dustbinallotment::DustbinAllotment};

pub struct BaseAllotmentRequest<T> {
    metadata: AllotmentMetadata,
    name: String,
    priority: i64,
    allotment: Mutex<Option<Arc<T>>>,
    coord_system: CoordinateSystem,
    depth: i8,
    max: Mutex<i64>
}

impl<T> BaseAllotmentRequest<T> {
    pub fn new(metadata: &AllotmentMetadata, coord_system: &CoordinateSystem, depth: i8) -> BaseAllotmentRequest<T> {
        BaseAllotmentRequest {
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

impl<T: AllotmentImpl + 'static> AllotmentRequestImpl for BaseAllotmentRequest<T> {
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

    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl> { self }
}

impl BaseAllotmentRequest<DustbinAllotment> {
    pub fn new_dustbin() -> BaseAllotmentRequest<DustbinAllotment> {
        BaseAllotmentRequest {
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
