use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{ Hash, Hasher }, sync::{Arc, Mutex}};

use crate::{AllotmentGroup, AllotmentPosition};

#[derive(Clone)]
pub struct AllotmentMetadataStore {
    metadata: Arc<Mutex<HashMap<String,AllotmentMetadata>>>
}

impl AllotmentMetadataStore {
    pub fn new() -> AllotmentMetadataStore {
        let out = AllotmentMetadataStore {
            metadata: Arc::new(Mutex::new(HashMap::new()))
        };
        out.add(AllotmentMetadataRequest::new("",0));
        out
    }

    pub fn add(&self, metadata: AllotmentMetadataRequest) {
        let mut allotments = self.metadata.lock().unwrap();
        if allotments.get(metadata.name()).is_none() {
            allotments.insert(metadata.name().to_string(),AllotmentMetadata::new(metadata));
        }
    }

    pub fn get(&self, name: &str) -> Option<AllotmentMetadata> {
        self.metadata.lock().unwrap().get(name).cloned()
    }
}

#[derive(Debug,Clone)]
pub struct AllotmentMetadataRequest {
    name: String,
    priority: i64,
    pairs: HashMap<String,String>
}

impl AllotmentMetadataRequest {
    pub fn dustbin() -> AllotmentMetadataRequest {
        AllotmentMetadataRequest::new("",0)
    }

    pub fn new(name: &str, priority: i64) -> AllotmentMetadataRequest {
        AllotmentMetadataRequest {
            name: name.to_string(),
            priority,
            pairs: HashMap::new()
        }
    }

    pub fn rebuild(metadata: &AllotmentMetadata) -> AllotmentMetadataRequest {
        let pairs = metadata.metadata.pairs.clone();
        AllotmentMetadataRequest {
            name: metadata.metadata.name.clone(),
            priority: metadata.metadata.priority,
            pairs
        }
    }

    pub fn add_pair(&mut self, key: &str, value: &str) {
        self.pairs.insert(key.to_string(),value.to_string());
    }

    fn hash_value(&self) -> u64 {
        let mut state = DefaultHasher::new();
        self.name.hash(&mut state);
        let mut pairs_key = self.pairs.keys().collect::<Vec<_>>();
        pairs_key.sort();
        for k in pairs_key {
            k.hash(&mut state);
            self.pairs.get(k).unwrap().hash(&mut state);
        }
        self.priority.hash(&mut state);
        state.finish()
    }

    pub fn name(&self) -> &str { &self.name }
    fn is_dustbin(&self) -> bool { self.name == "" }
    fn priority(&self) -> i64 { self.priority }
    fn summarize(&self) -> HashMap<String,String> { self.pairs.clone() }
}

#[derive(Clone,Debug)]
pub struct AllotmentMetadata {
    metadata: Arc<AllotmentMetadataRequest>,
    hash: u64
}

impl Hash for AllotmentMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentMetadata {
    fn eq(&self, other: &AllotmentMetadata) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentMetadata {}

impl AllotmentMetadata {
    pub(super) fn new(builder: AllotmentMetadataRequest) -> AllotmentMetadata {
        AllotmentMetadata {
            hash: builder.hash_value(),
            metadata: Arc::new(builder)
        }
    }

    pub fn name(&self) -> &str { &self.metadata.name }
    pub fn priority(&self) -> i64 { self.metadata.priority }
    pub fn is_dustbin(&self) -> bool { self.metadata.name == "" }

    pub fn allotment_group(&self) -> AllotmentGroup { 
        if self.metadata.name.starts_with("window:") {
            AllotmentGroup::Overlay(if self.metadata.name.ends_with("-over") { 1 } else { 0 })
        } else {
            AllotmentGroup::Track
        }
    }

    pub fn summarize(&self) -> HashMap<String,String> {
        self.metadata.summarize()
    }
}

#[derive(Clone,Debug)]
pub struct AllotmentMetadataReport {
    allotments: Arc<Vec<AllotmentMetadata>>,
    summary: Arc<Vec<HashMap<String,String>>>,
    hash: u64
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

impl AllotmentMetadataReport {
    pub fn new(allotments: Vec<AllotmentMetadata>) -> AllotmentMetadataReport {
        let mut summary = vec![];
        let mut state = DefaultHasher::new();
        for a in &allotments {
            summary.push(a.summarize());
            a.hash(&mut state);
        }
        AllotmentMetadataReport {
            allotments: Arc::new(allotments),
            summary: Arc::new(summary),
            hash: state.finish()
        }
    }

    pub fn summarize(&self) -> Arc<Vec<HashMap<String,String>>> { self.summary.clone() }
}
