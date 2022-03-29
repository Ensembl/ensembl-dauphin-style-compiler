use std::{sync::{Arc, Mutex}, ops::Range, cmp::Ordering, collections::HashMap};
use peregrine_toolkit::lock;

use peregrine_toolkit::watermark::Watermark;

use crate::allotment::style::allotmentname::AllotmentName;

use super::rangeused::RangeUsed;

struct Part {
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

struct CollisionAlgorithm {
    watermark: f64,
    parts: Vec<Part>,
    value: HashMap<AllotmentName,f64>
}

impl CollisionAlgorithm {
    fn new() -> CollisionAlgorithm {
        CollisionAlgorithm {
            watermark: 0.,
            parts: vec![],
            value: HashMap::new()
        }
    }

    #[cfg(any(text,debug_assertions))]
    #[allow(unused)]
    fn len(&self) -> usize { self.parts.len() }

    fn add_entry(&mut self, name: &AllotmentName, range: &RangeUsed<f64>, height: f64) {
        match range {
            RangeUsed::None => {
                self.value.insert(name.clone(),0.);
            },
            RangeUsed::All => {
                self.watermark += height;
                self.value.insert(name.clone(),self.watermark);
            },
            RangeUsed::Part(a,b) => {
                let interval = (*a as i64)..(*b as i64);
                self.parts.push(Part {
                    name: name.clone(),
                    interval,
                    height, 
                    tiebreak: self.parts.len()
                });
            }
        }
    }

    fn bump(&mut self) {
        /* sort parts into decreasing size order */
        self.parts.sort();
        self.parts.reverse();
        let mut watermark = Watermark::new();
        for part in &mut self.parts {
            let height = watermark.add(part.interval.start,part.interval.end,part.height) + self.watermark;
            self.value.insert(part.name.clone(),height);
        }
        self.watermark += watermark.max_height();
    }

    fn get(&self, name: &AllotmentName) -> f64 { self.value.get(name).cloned().unwrap_or(0.) }
    fn height(&self) -> f64 { self.watermark }
}

#[derive(Clone)]
pub struct CollisionAlgorithmHolder(Arc<Mutex<CollisionAlgorithm>>);

impl CollisionAlgorithmHolder {
    pub(crate) fn new() -> CollisionAlgorithmHolder {
        CollisionAlgorithmHolder(Arc::new(Mutex::new(CollisionAlgorithm::new())))
    }

    pub(crate) fn add_entry(&self, name: &AllotmentName, range: &RangeUsed<f64>, height: f64) {
        lock!(self.0).add_entry(name,range,height)
    }

    pub(crate) fn bump(&self) { lock!(self.0).bump() }
    pub(crate) fn height(&self) -> f64 { lock!(self.0).height() }
    pub(crate) fn get(&self, name: &AllotmentName) -> f64 { lock!(self.0).get(name) }

    #[cfg(any(text,debug_assertions))]
    #[allow(unused)]
    pub(crate) fn len(&self) -> usize { lock!(self.0).len() }
}
