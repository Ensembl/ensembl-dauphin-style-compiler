use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::lock;

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
    position: HashMap<(SymbolicAxis,String),DelayedValue>
}

impl Arbitrator {
    pub fn new() -> Arbitrator {
        Arbitrator {
            position: HashMap::new()
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
}
