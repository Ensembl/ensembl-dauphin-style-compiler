use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::{Arc, Mutex}, hash::{Hash, Hasher}};

use lru::LruCache;
use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleValue, PuzzleSolution}, lock};

#[derive(Clone)]
pub struct AllotmentMetadataGroup {
    values: HashMap<String,PuzzleValueHolder<String>>
}

impl AllotmentMetadataGroup {
    pub fn new(values: HashMap<String,PuzzleValueHolder<String>>) -> AllotmentMetadataGroup {
        AllotmentMetadataGroup {
            values
        }
    }

    pub fn get(&self, solution: &PuzzleSolution) -> HashMap<String,String> {
        self.values.iter().map(|(k,v)| (k.clone(),v.get(solution).as_ref().clone())).collect()
    }
}

#[derive(Clone,Debug)]
pub struct AllotmentMetadataReport2 {
    summary: Arc<Vec<HashMap<String,String>>>,
    hash: u64
}

impl AllotmentMetadataReport2 {
    pub fn empty() -> AllotmentMetadataReport2 {
        Self::new(vec![])
    }

    fn new(summary: Vec<HashMap<String,String>>) -> AllotmentMetadataReport2 {
        let mut state = DefaultHasher::new();
        for member in &summary {
            let mut keys = member.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                key.hash(&mut state);
                member.get(key).hash(&mut state);
            }
        }
        AllotmentMetadataReport2 {
            hash: state.finish(),
            summary: Arc::new(summary)
        }
    }

    pub fn summarize(&self) -> &Vec<HashMap<String,String>> {
        self.summary.as_ref()
    }
}

impl Hash for AllotmentMetadataReport2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentMetadataReport2 {
    fn eq(&self, other: &AllotmentMetadataReport2) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentMetadataReport2 {}


pub struct AllotmentMetadata2Builder {
    groups: Vec<AllotmentMetadataGroup>
}

impl AllotmentMetadata2Builder {
    pub fn new() -> AllotmentMetadata2Builder {
        AllotmentMetadata2Builder {
            groups: vec![]
        }
    }

    pub fn add(&mut self, group: AllotmentMetadataGroup) {
        self.groups.push(group);
    }

    pub fn union(&self, other: &AllotmentMetadata2Builder) -> AllotmentMetadata2Builder {
        let mut groups = self.groups.clone();
        groups.extend(other.groups.iter().cloned());
        AllotmentMetadata2Builder { groups }
    }
}

#[derive(Clone)]
pub struct AllotmentMetadata2 {
    groups: Arc<Vec<AllotmentMetadataGroup>>,
    cache: Arc<Mutex<LruCache<u64,AllotmentMetadataReport2>>>
}

impl AllotmentMetadata2 {
    pub fn new(builder: &AllotmentMetadata2Builder) -> AllotmentMetadata2 {
        AllotmentMetadata2 {
            groups: Arc::new(builder.groups.clone()),
            cache: Arc::new(Mutex::new(LruCache::new(16)))
        }
    }

    fn calculate(&self, solution: &PuzzleSolution) -> Vec<HashMap<String,String>> {
        self.groups.iter().map(|group| group.get(solution)).collect()
    }

    pub fn get(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport2 {
        let mut cache = lock!(self.cache);
        if let Some(cached) = cache.get(&solution.id()) {
            return cached.clone();
        }
        let data = AllotmentMetadataReport2::new(self.calculate(solution));
        cache.put(solution.id(),data.clone());
        data
    }
}
