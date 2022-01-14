use std::collections::HashMap;

/* The SecondaryPositionStore stores the offsets of other elements for alingment.
 */

pub struct SecondaryPositionStore {
    position: HashMap<String,i64>
}

impl SecondaryPositionStore {
    pub fn new() -> SecondaryPositionStore {
        SecondaryPositionStore {
            position: HashMap::new()
        }
    }

    pub fn lookup(&self, name: &str) -> Option<i64> { self.position.get(name).cloned() }

    pub fn add(&mut self, name: &str, offset: i64) {
        self.position.insert(name.to_string(),offset);
    }
}
