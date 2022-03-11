use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{ Hash, Hasher }, sync::{Arc, Mutex}};

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

    pub fn get_or_default(&self, name: &str) -> AllotmentMetadata {
        self.get(name).unwrap_or_else(|| AllotmentMetadata::new(AllotmentMetadataRequest::new(name,0)))
    }
}

#[derive(Debug,Clone)]
pub enum MetadataMergeStrategy {
    Replace,
    Keep,
    Minimum,
    Maximum
}

#[derive(Debug,Clone)]
struct MetadataValue {
    strategy: MetadataMergeStrategy,
    value: String    
}

impl MetadataValue {
    fn merge_numeric<F>(self,new: MetadataValue, use_new_pred: F) -> Option<MetadataValue> where F: FnOnce(f64,f64) -> bool {
        let old_number = self.value.parse::<f64>().ok();
        let new_number = new.value.parse::<f64>().ok();
        let use_new = match (old_number,new_number) {
            (Some(old),Some(new)) => use_new_pred(old,new),
            (Some(_),None) => false,
            (None,Some(_)) => true,
            (None,None) => { return None; }
        };
        if use_new { Some(new) } else { Some(self) }
    }

    fn merge(self, new: MetadataValue) -> Option<MetadataValue> {
        match self.strategy {
            MetadataMergeStrategy::Replace => Some(new),
            MetadataMergeStrategy::Keep => Some(self),
            MetadataMergeStrategy::Maximum => self.merge_numeric(new,|old,new| old < new),
            MetadataMergeStrategy::Minimum => self.merge_numeric(new,|old,new| old >= new)
        }
    }
}

#[derive(Debug,Clone)]
pub struct AllotmentMetadataRequest {
    name: String,
    priority: i64,
    pairs: HashMap<String,MetadataValue>
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

    pub fn merge(&mut self, other: &AllotmentMetadata) {
        for (key,value) in other.metadata.pairs.iter() {
            self.add_pair(key, &value.value,&value.strategy);
        }
    }

    pub fn add_pair(&mut self, key: &str, value: &str, merge: &MetadataMergeStrategy) {
        let new = MetadataValue { value: value.to_string(), strategy: merge.clone() };
        let value = if let Some(old) = self.pairs.remove(key) {
            old.merge(new)
        } else {
            Some(new)
        };
        if let Some(value) = value {
            self.pairs.insert(key.to_string(),value);
        }
    }

    fn hash_value(&self) -> u64 {
        let mut state = DefaultHasher::new();
        self.name.hash(&mut state);
        let mut pairs_key = self.pairs.keys().collect::<Vec<_>>();
        pairs_key.sort();
        for k in pairs_key {
            k.hash(&mut state);
            self.pairs.get(k).map(|x| &x.value).unwrap().hash(&mut state);
        }
        self.priority.hash(&mut state);
        state.finish()
    }

    pub fn name(&self) -> &str { &self.name }
    fn summarize(&self) -> HashMap<String,String> {
        self.pairs.iter().map(|(k,v)| (k.clone(),v.value.clone())).collect()
    }
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
    pub(crate) fn new(builder: AllotmentMetadataRequest) -> AllotmentMetadata {
        AllotmentMetadata {
            hash: builder.hash_value(),
            metadata: Arc::new(builder)
        }
    }

    pub(crate) fn empty() -> AllotmentMetadata {
        AllotmentMetadata::new(AllotmentMetadataRequest::dustbin())
    }

    pub fn name(&self) -> &str { &self.metadata.name }
    pub fn priority(&self) -> i64 { self.metadata.priority }

    pub fn get(&self, name: &str) -> Option<&String> { self.metadata.pairs.get(name).as_ref().map(|x| &x.value) }
    pub fn get_i64(&self, name: &str) -> Option<i64> { self.get(name).map(|x| x.parse().ok()).flatten() }
    pub fn get_f64(&self, name: &str) -> Option<f64> { self.get(name).map(|x| x.parse().ok()).flatten() }

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