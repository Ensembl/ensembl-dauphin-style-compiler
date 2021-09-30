use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

use crate::{Allotment, AllotmentGroup, AllotmentMetadata, DataMessage};

use super::{allotment::AllotmentImpl, allotmentrequest::{AllotmentRequestImpl}};

pub struct BaseAllotmentRequest<T> {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<T>>>,
    group: AllotmentGroup,
    max: Mutex<i64>
}

impl<T> BaseAllotmentRequest<T> {
    pub fn new(metadata: &AllotmentMetadata, group: &AllotmentGroup) -> BaseAllotmentRequest<T> {
        BaseAllotmentRequest { metadata: metadata.clone(), allotment: Mutex::new(None), group: group.clone(), max: Mutex::new(0) }
    }

    pub fn set_allotment(&self, value: Arc<T>) {
        *self.allotment.lock().unwrap() = Some(value);
    }

    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }
    pub fn max_used(&self) -> i64 { *self.max.lock().unwrap() }

    pub fn best_offset(&self, offset: i64) -> i64 {
        let padding_top = self.metadata.get_i64("padding-top").unwrap_or(0);
        offset + padding_top
    }

    pub fn best_height(&self) -> i64 {
        let mut height = self.max_used().max(0);
        if let Some(padding_top) = self.metadata.get_i64("padding-top") {
            height += padding_top;
        }
        if let Some(padding_bottom) = self.metadata.get_i64("padding-bottom") {
            height += padding_bottom;
        }
        if let Some(min_height) = self.metadata.get_i64("min-height") {
            if height < min_height {
                height = min_height;
            }
        }
        height
    }

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

    fn register_usage(&self, max: i64) {
        let mut self_max = lock!(self.max);
        *self_max = (*self_max).max(max)
    }

    fn up(self: Arc<Self>) -> Arc<dyn AllotmentRequestImpl> { self }
}
