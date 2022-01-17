use std::collections::HashMap;

/* The Arbitrator stores the offsets of other elements for alingment.
 */

 #[derive(Clone,PartialEq,Eq,Hash)]
pub enum SymbolicAxis {
    ScreenHoriz,
    ScreenVert
}

pub struct Arbitrator {
    position: HashMap<(SymbolicAxis,String),i64>
}

impl Arbitrator {
    pub fn new() -> Arbitrator {
        Arbitrator {
            position: HashMap::new()
        }
    }

    pub fn lookup_symbolic(&self, axis: &SymbolicAxis, name: &str) -> Option<i64> {
        self.position.get(&(axis.clone(),name.to_string())).cloned()
    }

    pub fn add_symbolic(&mut self, axis: &SymbolicAxis, name: &str, offset: i64) {
        self.position.insert((axis.clone(),name.to_string()),offset);
    }
}
