use std::{sync::{Arc, Mutex}, ops::Range, cmp::Ordering};
use peregrine_toolkit::lock;

use crate::allotment::core::rangeused::RangeUsed;
use peregrine_toolkit::watermark::Watermark;
use lazy_static::lazy_static;
use identitynumber::identitynumber;

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

identitynumber!(IDS);

struct CollisionAlgorithm {
    bumped: bool,
    all_wm: f64,
    parts: Vec<Part>,
    xxx_tokens: Vec<CollisionToken>,
    id: u64
}

impl CollisionAlgorithm {
    fn new() -> CollisionAlgorithm {
        CollisionAlgorithm {
            bumped: false,
            all_wm: 0.,
            parts: vec![],
            xxx_tokens: vec![],
            id: IDS.next()
        }
    }

    fn add_entry(&mut self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        match range {
            RangeUsed::None => { CollisionToken::new(0.) },
            RangeUsed::All => {
                self.all_wm += height;
                CollisionToken::new(self.all_wm)
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
                self.xxx_tokens.push(token.clone());
                token
            }
        }
    }

    fn bump(&mut self) {
        use web_sys::console;
        console::log_1(&format!("bumping {}!",self.id).into());
        if self.bumped {
            console::log_1(&format!("ALREADY BUMPED {}!",self.id).into());
        }
        self.bumped = true;
        /* sort parts into decreasing size order */
        self.parts.sort();
        self.parts.reverse();
        let mut watermark = Watermark::new();
        for part in &mut self.parts {
            part.token.set(watermark.add(part.interval.start,part.interval.end,part.height) + self.all_wm);
        }
        self.all_wm += watermark.max_height();
        console::log_1(&format!("bumping {:?} {:?} all_wm={}",self.xxx_tokens,self.parts,self.all_wm).into());
    }
}

identitynumber!(IDS2);

#[derive(Clone)]
pub struct CollisionAlgorithmHolder(Arc<Mutex<CollisionAlgorithm>>,u64);

impl CollisionAlgorithmHolder {
    pub(crate) fn new() -> CollisionAlgorithmHolder {
        CollisionAlgorithmHolder(Arc::new(Mutex::new(CollisionAlgorithm::new())),IDS2.next())
    }

    pub(crate) fn add_entry(&self, range: &RangeUsed<f64>, height: f64) -> CollisionToken {
        lock!(self.0).add_entry(range,height)
    }

    pub(crate) fn bump(&self) {
        use web_sys::console;
        console::log_1(&format!("bumping holder {}",self.1).into());
        lock!(self.0).bump();
    }
}
