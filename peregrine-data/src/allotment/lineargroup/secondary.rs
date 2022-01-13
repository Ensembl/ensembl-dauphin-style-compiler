use std::collections::HashMap;

/* The SecondaryPositionStore stores the offsets of other elements for alingment.
 */

pub struct SecondaryPosition {
    pub offset: i64,
    pub size: i64,
    pub reverse: bool
}

pub struct SecondaryPositionStore {
    position: HashMap<String,SecondaryPosition>
}

impl SecondaryPositionStore {
    pub fn new() -> SecondaryPositionStore {
        SecondaryPositionStore {
            position: HashMap::new()
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&SecondaryPosition> {
        self.position.get(name)
    }

    pub fn add(&mut self, name: &str, offset: i64, size: i64, reverse: bool) {
        self.position.insert(name.to_string(),SecondaryPosition { offset, size, reverse });
    }
}
