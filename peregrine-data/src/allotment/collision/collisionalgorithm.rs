use std::{sync::{Arc, Mutex}, collections::HashMap};
use peregrine_toolkit::lock;
use peregrine_toolkit::skyline::Skyline;
use crate::allotment::{style::allotmentname::AllotmentName, util::rangeused::RangeUsed};
use super::bumppart::Part;

struct CollisionAlgorithmData {
    watermark: f64,
    parts: Vec<Part>,
    value: HashMap<AllotmentName,f64>
}

impl CollisionAlgorithmData {
    fn new() -> CollisionAlgorithmData {
        CollisionAlgorithmData {
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
                self.parts.push(Part::new(name,&interval,height,self.parts.len()));
            }
        }
    }

    fn bump(&mut self) {
        /* sort parts into decreasing size order */
        self.parts.sort();
        self.parts.reverse();
        let mut watermark = Skyline::new();
        for part in &mut self.parts {
            let height = part.watermark_add(&mut watermark) + self.watermark;
            self.value.insert(part.name().clone(),height);
        }
        self.watermark += watermark.max_height();
    }

    fn get(&self, name: &AllotmentName) -> f64 { self.value.get(name).cloned().unwrap_or(0.) }
    fn height(&self) -> f64 { self.watermark }
}

#[derive(Clone)]
pub struct CollisionAlgorithm(Arc<Mutex<CollisionAlgorithmData>>);

impl CollisionAlgorithm {
    pub(crate) fn new() -> CollisionAlgorithm {
        CollisionAlgorithm(Arc::new(Mutex::new(CollisionAlgorithmData::new())))
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
