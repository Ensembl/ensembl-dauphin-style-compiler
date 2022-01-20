use std::{collections::HashMap, sync::{Arc, Mutex}};

use peregrine_toolkit::lock;

/* The Arbitrator stores the offsets of other elements for alingment.
 */

pub struct DelayedValue {
    source: Arc<Mutex<i64>>,
    callback: Box<dyn Fn(i64) -> i64>
}

impl DelayedValue {
    pub fn new<F>(source: &Arc<Mutex<i64>>, callback: F) -> DelayedValue where F: Fn(i64) -> i64 + 'static {
        DelayedValue {
            source: source.clone(),
            callback: Box::new(callback)
        }
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

    pub fn lookup_symbolic(&self, axis: &SymbolicAxis, name: &str) -> Option<i64> {
        self.position.get(&(axis.clone(),name.to_string())).map(|x| x.value())
    }

    pub fn add_symbolic(&mut self, axis: &SymbolicAxis, name: &str, value: DelayedValue) {
        self.position.insert((axis.clone(),name.to_string()),value);
    }
}
