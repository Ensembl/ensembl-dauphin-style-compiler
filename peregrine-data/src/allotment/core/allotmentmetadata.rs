use std::{collections::{HashMap, hash_map::DefaultHasher}, sync::{Arc, Mutex}, hash::{Hash, Hasher}};

use lru::LruCache;
use peregrine_toolkit::{lock, puzzle::{ StaticValue, StaticAnswer}};

#[derive(Clone)]
pub struct AllotmentMetadataGroup {
    values: HashMap<String,StaticValue<String>>
}

impl AllotmentMetadataGroup {
    pub fn new(values: HashMap<String,StaticValue<String>>) -> AllotmentMetadataGroup {
        AllotmentMetadataGroup {
            values
        }
    }

    pub fn get(&self, answer_index: &mut StaticAnswer) -> HashMap<String,String> {
        self.values.iter().map(|(k,v)| (k.clone(),v.call(answer_index).clone())).collect()
    }
}

#[derive(Clone,Debug)]
pub struct AllotmentMetadataReport {
    summary: Arc<Vec<HashMap<String,String>>>,
    hash: u64
}

impl AllotmentMetadataReport {
    pub fn empty() -> AllotmentMetadataReport {
        Self::new(vec![])
    }

    fn new(summary: Vec<HashMap<String,String>>) -> AllotmentMetadataReport {
        let mut state = DefaultHasher::new();
        for member in &summary {
            let mut keys = member.keys().collect::<Vec<_>>();
            keys.sort();
            for key in keys {
                key.hash(&mut state);
                member.get(key).hash(&mut state);
            }
        }
        AllotmentMetadataReport {
            hash: state.finish(),
            summary: Arc::new(summary)
        }
    }

    pub fn summarize(&self) -> &Vec<HashMap<String,String>> {
        self.summary.as_ref()
    }
}

impl Hash for AllotmentMetadataReport {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentMetadataReport {
    fn eq(&self, other: &AllotmentMetadataReport) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentMetadataReport {}


pub struct AllotmentMetadataBuilder {
    groups: Vec<AllotmentMetadataGroup>
}

impl AllotmentMetadataBuilder {
    pub fn new() -> AllotmentMetadataBuilder {
        AllotmentMetadataBuilder {
            groups: vec![]
        }
    }

    pub fn add(&mut self, group: AllotmentMetadataGroup) {
        self.groups.push(group);
    }
}

#[derive(Clone)]
pub struct AllotmentMetadata {
    groups: Arc<Vec<AllotmentMetadataGroup>>,
    cache: Arc<Mutex<LruCache<u64,AllotmentMetadataReport>>>
}

impl AllotmentMetadata {
    pub fn new(builder: &AllotmentMetadataBuilder) -> AllotmentMetadata {
        AllotmentMetadata {
            groups: Arc::new(builder.groups.clone()),
            cache: Arc::new(Mutex::new(LruCache::new(16)))
        }
    }

    fn calculate(&self, answer_index: &mut StaticAnswer) -> Vec<HashMap<String,String>> {
        self.groups.iter().map(|group| group.get(answer_index)).collect()
    }

    pub fn get(&self, answer_index: &mut StaticAnswer) -> AllotmentMetadataReport {
        let mut cache = lock!(self.cache);
        if let Some(cached) = cache.get(&answer_index.serial()) {
            return cached.clone();
        }
        let data = AllotmentMetadataReport::new(self.calculate(answer_index));
        cache.put(answer_index.serial(),data.clone());
        data
    }
}
