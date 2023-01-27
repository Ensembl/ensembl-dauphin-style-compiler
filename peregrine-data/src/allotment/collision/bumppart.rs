use std::{ops::Range};
use peregrine_toolkit::skyline::Skyline;
use crate::allotment::core::allotmentname::AllotmentName;

pub(super) struct Part {
    name: AllotmentName,
    interval: Range<i64>,
    height: f64
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Part({}-{}:{})",self.interval.start,self.interval.end,self.height)
    }
}

impl Part {
    pub(super) fn new(name: &AllotmentName, interval: &Range<i64>,  height: f64) -> Part {
        Part {
            name: name.clone(), 
            interval: interval.clone(),
            height
        }
    }

    pub(super) fn name(&self) -> &AllotmentName { &self.name }

    pub(super) fn shape(&self) -> (i64,i64,f64) {
        (self.interval.start,self.interval.end,self.height)
    }
}
