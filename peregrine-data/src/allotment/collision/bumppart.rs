use std::{cmp::Ordering, ops::Range};

use peregrine_toolkit::skyline::Skyline;

use crate::allotment::style::allotmentname::AllotmentName;

pub(super) struct Part {
    name: AllotmentName,
    tiebreak: usize,
    interval: Range<i64>,
    height: f64
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Part{}({}-{}:{})",self.tiebreak,self.interval.start,self.interval.end,self.height)
    }
}

impl Eq for Part {}

impl PartialEq for Part {
    fn eq(&self, other: &Self) -> bool {
        self.tiebreak == other.tiebreak
    }
}

impl PartialOrd for Part {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Part {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_size = self.interval.end - self.interval.start;
        let other_size = other.interval.end - other.interval.start;
        if self_size > other_size { return Ordering::Greater; }
        else if  self_size < other_size { return Ordering::Less; }
        if self.interval.start > other.interval.start { return Ordering::Greater; }
        if self.interval.start < other.interval.start { return Ordering::Less; }
        self.tiebreak.cmp(&other.tiebreak)
    }
}

impl Part {
    pub(super) fn new(name: &AllotmentName, interval: &Range<i64>,  height: f64, tiebreak: usize) -> Part {
        Part {
            name: name.clone(), 
            interval: interval.clone(),
            tiebreak, 
            height
        }
    }

    pub(super) fn name(&self) -> &AllotmentName { &self.name }

    pub(super) fn watermark_add(&self, watermark: &mut Skyline) -> f64 {
        watermark.add(self.interval.start,self.interval.end,self.height)
    }
}
