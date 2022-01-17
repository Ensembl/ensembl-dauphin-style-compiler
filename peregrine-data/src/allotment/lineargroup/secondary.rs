use std::collections::HashMap;

/* The SecondaryPositionResolver stores the offsets of other elements for alingment.
 */

pub struct SecondaryPositionResolver {
    position: HashMap<String,i64>
}

impl SecondaryPositionResolver {
    pub fn new() -> SecondaryPositionResolver {
        SecondaryPositionResolver {
            position: HashMap::new()
        }
    }

    pub fn lookup(&self, name: &str) -> Option<i64> { self.position.get(name).cloned() }

    pub fn add(&mut self, name: &str, offset: i64) {
        self.position.insert(name.to_string(),offset);
    }
}
