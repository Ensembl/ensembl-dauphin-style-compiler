use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::lock;

use crate::CarriageExtent;

use super::rangeused::RangeUsed;

/* The Arbitrator stores the offsets of other elements for alingment.
 */

#[derive(Clone)]
pub struct DelayedValue {
    source: Arc<Mutex<i64>>,
    callback: Arc<Box<dyn Fn(i64) -> i64>>
}

impl DelayedValue {
    pub fn new<F>(source: &Arc<Mutex<i64>>, callback: F) -> DelayedValue where F: Fn(i64) -> i64 + 'static {
        DelayedValue {
            source: source.clone(),
            callback: Arc::new(Box::new(callback))
        }
    }

    pub fn fixed(value: i64) -> DelayedValue {
        DelayedValue::new(&Arc::new(Mutex::new(value)),|x| x)
    }

    pub fn derived<F>(source: &DelayedValue, callback: F) -> DelayedValue where F: Fn(i64) -> i64 + 'static {
        let source_callback = source.callback.clone();
        DelayedValue {
            source: source.source.clone(),
            callback: Arc::new(Box::new(move |x| callback((source_callback)(x))))
        }
    }

    pub fn set_value(&self, value: i64) {
        *lock!(self.source) = value;
    }

    pub fn value(&self) -> i64 {
        (self.callback)(*lock!(self.source))
    }
}

 #[derive(Clone,PartialEq,Eq,Hash)]
pub enum SymbolicAxis {
    ScreenHoriz,
    ScreenVert
}

pub struct Arbitrator {
    position: HashMap<(SymbolicAxis,String),DelayedValue>,
    max_px_per_bp: Option<f64>,
    bp_start: f64
}

impl Arbitrator {
    fn real_calc_max_px_per_bp(extent: &CarriageExtent) -> f64 {
        let bp_per_carriage = extent.train().scale().bp_in_carriage() as f64;
        let max_px_per_carriage = extent.train().pixel_size().max_px_per_carriage() as f64;
        max_px_per_carriage / bp_per_carriage
    }

    fn calc_max_px_per_bp(extent: Option<&CarriageExtent>) -> Option<f64> {
        extent.map(|e| Arbitrator::real_calc_max_px_per_bp(e))
    }

    pub fn new(extent: Option<&CarriageExtent>) -> Arbitrator {
        Arbitrator {
            position: HashMap::new(),
            max_px_per_bp: Arbitrator::calc_max_px_per_bp(extent),
            bp_start: extent.map(|x| x.left_right().0).unwrap_or(0.)
        }
    }

    pub fn lookup_symbolic_delayed(&self, axis: &SymbolicAxis, name: &str) -> Option<&DelayedValue> {
        self.position.get(&(axis.clone(),name.to_string()))
    }

    pub fn lookup_symbolic(&self, axis: &SymbolicAxis, name: &str) -> Option<i64> {
        self.lookup_symbolic_delayed(axis,name).map(|x| x.value())
    }

    pub fn add_symbolic(&mut self, axis: &SymbolicAxis, name: &str, value: DelayedValue) {
        self.position.insert((axis.clone(),name.to_string()),value);
    }

    pub fn full_pixel_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>) -> RangeUsed<f64> {
        if let Some(max_px_per_bp) = self.max_px_per_bp {
            base_range.plus_scalar(-self.bp_start).pixel_range(pixel_range,max_px_per_bp)
        } else {
            pixel_range.clone()
        }
    }
}
