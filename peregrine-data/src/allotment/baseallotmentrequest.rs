use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use crate::{Allotment, AllotmentMetadata, DataMessage};
use super::{allotment::{AllotmentImpl, CoordinateSystem}, allotmentrequest::{AllotmentRequestImpl}, basicallotmentspec::BasicAllotmentSpec};

pub struct BaseAllotmentRequest<T> {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<T>>>,
    coord_system: CoordinateSystem,
    depth: i8,
    max: Mutex<i64>
}

impl<T> BaseAllotmentRequest<T> {
    pub fn new(metadata: &AllotmentMetadata, coord_system: &CoordinateSystem, depth: i8) -> BaseAllotmentRequest<T> {
        BaseAllotmentRequest {
            metadata: metadata.clone(),
            allotment: Mutex::new(None),
            depth,
            coord_system: coord_system.clone(),
            max: Mutex::new(0)
        }
    }

    pub fn set_allotment(&self, value: Arc<T>) {
        *self.allotment.lock().unwrap() = Some(value);
    }

    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }
    pub fn max_used(&self) -> i64 { *self.max.lock().unwrap() }

    pub fn base_allotment(&self) -> Option<Arc<T>> {
        self.allotment.lock().unwrap().as_ref().cloned()
    }
}

impl<T: AllotmentImpl + 'static> AllotmentRequestImpl for BaseAllotmentRequest<T> {
    fn name(&self) -> String {
        BasicAllotmentSpec::from_spec(self.metadata.name()).name().to_string()
    }

    fn is_dustbin(&self) -> bool { false }
    fn priority(&self) -> i64 { self.metadata.priority() }
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
