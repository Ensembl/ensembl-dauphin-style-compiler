use std::{collections::{HashSet, hash_map::DefaultHasher}, hash::{Hasher, Hash}};

use super::trackconfig::TrackConfigNode;

/* A SwitchOverlay contains those values which are explicitly added to a track as part of the
 * track config process rather than by the integration (using this object's set and clear methods).
 * These are then applied to a strack config with apply(). 
 * 
 * set_parents contain paths which need to be turthy for the setting to be visible. full_set contains
 * the explicitly set values.
 */

fn hash_path(data: &[&str]) -> u64 {
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish()
}

#[derive(Debug)]
pub(crate) struct SwitchOverlay {
    full_set: HashSet<Vec<String>>,
    set_parents: HashSet<u64>
}

impl SwitchOverlay {
    pub(crate) fn new() -> SwitchOverlay {
        SwitchOverlay {
            full_set: HashSet::new(),
            set_parents: HashSet::new()
        }
    }

    fn ensure_parents(&mut self, path: &[&str]) {
        for i in 0..path.len()-1 {
            self.set_parents.insert(hash_path(&path[0..i]));
        }
    }

    pub(crate) fn set(&mut self, path: &[&str]) {
        self.ensure_parents(path);
        self.full_set.insert(path.iter().map(|x| x.to_string()).collect());
    }

    pub(crate) fn clear(&mut self, path: &[&str]) {
        self.full_set.remove(&path.iter().map(|x| x.to_string()).collect::<Vec<_>>());
    }

    pub(super) fn apply(&self, node: &mut TrackConfigNode) {
        for path in &self.full_set {
            node.add_path(&path.iter().map(|x| x.as_str()).collect::<Vec<_>>());
        }
    }
}
