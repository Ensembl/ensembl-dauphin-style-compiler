use std::{sync::{Arc, Mutex}, ops::Range, cmp::Ordering};
use peregrine_toolkit::lock;

use crate::allotment::core::rangeused::RangeUsed;
use peregrine_toolkit::watermark::Watermark;

pub(crate) enum CollisionToken {
    All(usize),
    Part(usize),
    None
}

#[cfg_attr(debug_assertions,derive(Debug))]
struct Part {
    tiebreak: usize,
    interval: Range<i64>,
    offset: Option<f64>,
    height: f64
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
    all_wm: f64,
    alls: Vec<f64>,
    parts: Vec<Part>
}

impl CollisionAlgorithm {
    fn new() -> CollisionAlgorithm {
        CollisionAlgorithm {
            all_wm: 0.,
            alls: vec![],
            parts: vec![]
        }
    }

    fn add_entry(&mut self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        match range {
            RangeUsed::None => { CollisionToken::None },
            RangeUsed::All => {
                self.alls.push(self.all_wm);
                self.all_wm += height;
                CollisionToken::All(&self.alls.len()-1)
            },
            RangeUsed::Part(a,b) => {
                let interval = (*a as i64)..(*b as i64);
                self.parts.push(Part { interval, offset: None, height, tiebreak: self.parts.len() });
                CollisionToken::Part(&self.parts.len()-1)
            }
        }
    }

    fn position(&self, token: &CollisionToken) -> f64 {
        match token {
            CollisionToken::All(offset) => self.alls[*offset],
            CollisionToken::Part(offset) => self.parts[*offset].offset.unwrap_or(0.),
            CollisionToken::None => 0.
        }
    }

    fn bump(&mut self) {
        /* sort parts into decreasing size order */
        self.parts.sort();
        self.parts.reverse();
        let mut watermark = Watermark::new();
        for part in &mut self.parts {
            part.offset = Some(watermark.add(part.interval.start,part.interval.end,part.height) + self.all_wm);
        }
        self.all_wm += watermark.max_height();
        use web_sys::console;
        console::log_1(&format!("bumping {:?} {:?}",self.alls,self.parts).into());
    }
}

#[derive(Clone)]
pub struct CollisionAlgorithmHolder(Arc<Mutex<CollisionAlgorithm>>);

impl CollisionAlgorithmHolder {
    pub(super) fn new() -> CollisionAlgorithmHolder {
        CollisionAlgorithmHolder(Arc::new(Mutex::new(CollisionAlgorithm::new())))
    }

    pub(super) fn add_entry(&self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        lock!(self.0).add_entry(range,height)
    }

    pub(super) fn position(&self, token: &CollisionToken) -> f64 {
        lock!(self.0).position(token)
    }

    pub(super) fn bump(&self) {
        lock!(self.0).bump();
    }
}
