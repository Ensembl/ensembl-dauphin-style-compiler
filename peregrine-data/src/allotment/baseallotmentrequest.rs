use std::sync::{Arc, Mutex};

use crate::{Allotment, AllotmentDirection, AllotmentGroup, AllotmentMetadata, DataMessage};

use super::{allotment::AllotmentImpl, allotmentrequest::{AllotmentRequestImpl}};

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
    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl> { self }
}
