use std::collections::{ BTreeMap, HashSet, HashMap };
use crate::webgl::FlatId;

pub struct EvictList {
    map: BTreeMap<i64,HashSet<FlatId>>,
    revmap: HashMap<FlatId,i64>
}

impl EvictList {
    pub fn new() -> EvictList {
        EvictList {
            map: BTreeMap::new(),
            revmap: HashMap::new()
        }
    }

    pub fn insert(&mut self, flat_id: &FlatId, when: i64) {
        if let Some(old_when) = self.revmap.get(flat_id) {
            self.map.get_mut(old_when).unwrap().remove(flat_id);
        }
        self.map.entry(when).or_insert_with(|| HashSet::new()).insert(flat_id.clone());
        self.revmap.insert(flat_id.clone(), when);
    }

    pub fn remove_item(&mut self, flat_id: &FlatId) -> bool {
        let mut del = None;
        let mut found = false;
        if let Some(when) = self.revmap.remove(flat_id) {
            let group = self.map.get_mut(&when).unwrap();
            group.remove(flat_id);
            if group.len() == 0 {
                del = Some(when);
            }
            found = true;
        }
        if let Some(epoch) = del {
            self.map.remove(&epoch);
        }
        found
    }

    pub fn remove_oldest(&mut self) -> Option<(i64,FlatId)> {
        let mut del = None;
        let mut out = None;
        if let Some((epoch,group)) = self.map.iter_mut().next() {
            let victim = group.iter().next().unwrap().clone();
            group.remove(&victim);
            if group.len() == 0 {
                del = Some(*epoch);
            }
            out = Some((*epoch,victim.clone()));
        }
        if let Some(epoch) = del {
            self.map.remove(&epoch);
        }
        if let Some((_,victim)) = &out {
            self.revmap.remove(victim);
        }
        out
    }
}
