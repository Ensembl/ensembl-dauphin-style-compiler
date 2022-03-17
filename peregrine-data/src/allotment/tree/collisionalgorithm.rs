use std::{sync::{Arc, Mutex}, ops::Range, cmp::Ordering};
use peregrine_toolkit::lock;

use crate::allotment::core::rangeused::RangeUsed;
use peregrine_toolkit::watermark::Watermark;

#[derive(Clone)]
pub(crate) struct CollisionToken(Arc<Mutex<f64>>);

#[cfg(debug_assertions)]
impl std::fmt::Debug for CollisionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",*lock!(self.0))
    }
}

impl CollisionToken {
    fn new(value: f64) -> CollisionToken {
        CollisionToken(Arc::new(Mutex::new(value)))
    }

    fn set(&self, value: f64) { *lock!(self.0) = value; }
    pub fn get(&self) -> f64 { *lock!(self.0) }
}

struct Part {
    tiebreak: usize,
    interval: Range<i64>,
    height: f64,
    token: CollisionToken
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Part{}({}-{}:{})={:?}",self.tiebreak,self.interval.start,self.interval.end,self.height,self.token)
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
    parts: Vec<Part>
}

impl CollisionAlgorithm {
    fn new() -> CollisionAlgorithm {
        CollisionAlgorithm {
            watermark: 0.,
            parts: vec![]
        }
    }

    fn len(&self) -> usize { self.parts.len() }

    fn add_entry(&mut self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        match range {
            RangeUsed::None => { CollisionToken::new(0.) },
            RangeUsed::All => {
                self.watermark += height;
                CollisionToken::new(self.watermark)
            },
            RangeUsed::Part(a,b) => {
                let interval = (*a as i64)..(*b as i64);
                let token = CollisionToken::new(0.);
                self.parts.push(Part {
                    interval,
                    height, 
                    tiebreak: self.parts.len(),
                    token: token.clone()
                });
                token
            }
        }
    }

    fn bump(&mut self) -> f64 {
        /* sort parts into decreasing size order */
        self.parts.sort();
        self.parts.reverse();
        let mut watermark = Watermark::new();
        for part in &mut self.parts {
            part.token.set(watermark.add(part.interval.start,part.interval.end,part.height) + self.watermark);
        }
        watermark.max_height()
    }
}

#[derive(Clone)]
pub struct CollisionAlgorithmHolder(Arc<Mutex<CollisionAlgorithm>>);

impl CollisionAlgorithmHolder {
    pub(crate) fn new() -> CollisionAlgorithmHolder {
        CollisionAlgorithmHolder(Arc::new(Mutex::new(CollisionAlgorithm::new())))
    }

    pub(crate) fn add_entry(&self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        lock!(self.0).add_entry(range,height)
    }

    pub(crate) fn bump(&self) -> f64 {
        lock!(self.0).bump()
    }

    pub(crate) fn len(&self) -> usize { lock!(self.0).len() }
}
