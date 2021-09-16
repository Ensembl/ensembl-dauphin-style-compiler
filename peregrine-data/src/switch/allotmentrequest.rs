use crate::AllotmentPositionKind;
use std::hash::Hasher;
use std::hash::Hash;
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;

#[derive(Debug)]
pub struct AllotmentRequestBuilder {
    name: String,
    priority: i64,
    pairs: HashMap<String,String>
}

impl AllotmentRequestBuilder {
    pub fn dustbin() -> AllotmentRequestBuilder {
        AllotmentRequestBuilder::new("",0)
    }

    pub fn new(name: &str, priority: i64) -> AllotmentRequestBuilder {
        AllotmentRequestBuilder {
            name: name.to_string(),
            priority,
            pairs: HashMap::new()
        }
    }

    pub fn rebuild(metadata: &AllotmentRequest) -> AllotmentRequestBuilder {
        let pairs = metadata.metadata.pairs.clone();
        AllotmentRequestBuilder {
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

    fn priority(&self) -> i64 { self.priority }
    fn summarize(&self) -> HashMap<String,String> { self.pairs.clone() }
}

#[derive(Clone,Debug)]
pub struct AllotmentRequest {
    metadata: Arc<AllotmentRequestBuilder>,
    hash: u64
}

impl Hash for AllotmentRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentRequest {
    fn eq(&self, other: &AllotmentRequest) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentRequest {}

impl AllotmentRequest {
    pub fn new(builder: AllotmentRequestBuilder) -> AllotmentRequest {
        AllotmentRequest {
            hash: builder.hash_value(),
            metadata: Arc::new(builder)
        }
    }

    pub fn name(&self) -> &str { &self.metadata.name }
    pub fn priority(&self) -> i64 { self.metadata.priority }
    pub fn is_dustbin(&self) -> bool { self.metadata.name == "" }

    pub fn kind(&self) -> AllotmentPositionKind { 
        if self.metadata.name.starts_with("window:") {
            AllotmentPositionKind::Overlay(if self.metadata.name.ends_with("-over") { 1 } else { 0 })
        } else {
            AllotmentPositionKind::Track
        }
    }

    pub fn summarize(&self) -> HashMap<String,String> {
        self.metadata.summarize()
    }
}
