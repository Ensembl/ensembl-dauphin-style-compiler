use std::sync::{Arc, Mutex};
use peregrine_toolkit::lock;
use crate::{Allotment, AllotmentDirection, AllotmentMetadata, DataMessage, shape::shape::FilterMinMax};
use super::{allotment::{AllotmentImpl, CoordinateSystem}, allotmentrequest::{AllotmentRequestImpl}};

pub(super) fn remove_depth(spec: &mut String) -> i8 {
    let mut depth = 0;
    if let Some(start) = spec.find("[") {
        if let Some(end) = spec[start..].find("]").map(|x| x+start) {
            if let Some(new_depth) = spec[(start+1)..end].parse::<i8>().ok() {
                depth = new_depth;
                let mut new_spec = spec[0..start].to_string();
                new_spec.push_str(&spec[end+1..].to_string());
                *spec = new_spec;
            }
        }
    }
    depth
}


pub struct BaseAllotmentRequest<T> {
    metadata: AllotmentMetadata,
    allotment: Mutex<Option<Arc<T>>>,
    coord_system: CoordinateSystem,
    direction: AllotmentDirection,
    max: Mutex<i64>
}

impl<T> BaseAllotmentRequest<T> {
    pub fn new(metadata: &AllotmentMetadata, coord_system: &CoordinateSystem, direction: &AllotmentDirection) -> BaseAllotmentRequest<T> {
        BaseAllotmentRequest {
            metadata: metadata.clone(),
            allotment: Mutex::new(None),
            direction: direction.clone(),
            coord_system: coord_system.clone(),
            max: Mutex::new(0)
        }
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
    fn direction(&self) -> AllotmentDirection { self.direction.clone() }
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

    fn coord_system(&self) -> CoordinateSystem { self.coord_system.clone() }
}
