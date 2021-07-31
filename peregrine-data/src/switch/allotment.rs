use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::Hasher, sync::{ Arc, Mutex }};
use std::hash::{ Hash };
use keyed::{ keyed_handle, KeyedValues, KeyedHandle };

#[derive(Debug)]
pub struct AllotmentStaticMetadataBuilder {
    name: String,
    pairs: HashMap<String,String>
}

impl AllotmentStaticMetadataBuilder {
    pub fn dustbin() -> AllotmentStaticMetadataBuilder {
        AllotmentStaticMetadataBuilder::new("")
    }

    pub fn new(name: &str) -> AllotmentStaticMetadataBuilder {
        AllotmentStaticMetadataBuilder {
            name: name.to_string(),
            pairs: HashMap::new()
        }
    }

    pub fn rebuild(metadata: &AllotmentStaticMetadata) -> AllotmentStaticMetadataBuilder {
        let pairs = metadata.metadata.pairs.clone();
        AllotmentStaticMetadataBuilder {
            name: metadata.metadata.name.clone(),
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
        state.finish()
    }    

    fn summarize(&self) -> HashMap<String,String> {
        self.pairs.clone()
    }
}

#[derive(Clone,Debug)]
pub struct AllotmentStaticMetadata {
    metadata: Arc<AllotmentStaticMetadataBuilder>,
    hash: u64
}

impl Hash for AllotmentStaticMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotmentStaticMetadata {
    fn eq(&self, other: &AllotmentStaticMetadata) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotmentStaticMetadata {}

impl AllotmentStaticMetadata {
    fn new(builder: AllotmentStaticMetadataBuilder) -> AllotmentStaticMetadata {
        AllotmentStaticMetadata {
            hash: builder.hash_value(),
            metadata: Arc::new(builder)
        }
    }

    pub fn name(&self) -> &str { &self.metadata.name }
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

#[derive(Clone,Debug)]
pub struct AllotterMetadata {
    allotments: Arc<Vec<AllotmentStaticMetadata>>,
    summary: Arc<Vec<HashMap<String,String>>>,
    hash: u64
}

impl Hash for AllotterMetadata {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for AllotterMetadata {
    fn eq(&self, other: &AllotterMetadata) -> bool {
        self.hash == other.hash
    }
}

impl Eq for AllotterMetadata {}

impl AllotterMetadata {
    pub fn new(allotments: Vec<AllotmentStaticMetadata>) -> AllotterMetadata {
        let mut summary = vec![];
        let mut state = DefaultHasher::new();
        for a in &allotments {
            summary.push(a.summarize());
            a.hash(&mut state);
        }
        AllotterMetadata {
            allotments: Arc::new(allotments),
            summary: Arc::new(summary),
            hash: state.finish()
        }
    }

    pub fn summarize(&self) -> Arc<Vec<HashMap<String,String>>> { self.summary.clone() }
}

#[derive(Clone,Debug)]
pub struct AllotmentRequest {
    metadata: AllotmentStaticMetadata,
    priority: i64
}

impl AllotmentRequest {
    pub fn new(metadata: AllotmentStaticMetadataBuilder, priority: i64) -> AllotmentRequest {
        AllotmentRequest {
            metadata: AllotmentStaticMetadata::new(metadata),
            priority
        }
    }

    pub fn merge(&mut self, other: &AllotmentRequest) {
        if self.metadata().name() != other.metadata().name() { return; }
        if other.priority < self.priority {
            self.priority = other.priority;
        }
    }

    pub fn priority(&self) -> i64 { self.priority }
    pub fn metadata(&self) -> &AllotmentStaticMetadata { &self.metadata }
}


keyed_handle!(AllotmentHandle);

impl AllotmentHandle {
    pub fn is_null(&self) -> bool { self.get() == 0 }
}

#[derive(Clone)]
pub struct AllotmentPetitioner {
    allotments: Arc<Mutex<KeyedValues<AllotmentHandle,AllotmentRequest>>>,
}

impl AllotmentPetitioner {
    pub fn new() -> AllotmentPetitioner {
        let mut out = AllotmentPetitioner {
            allotments: Arc::new(Mutex::new(KeyedValues::new()))
        };
        out.add(AllotmentRequest::new(AllotmentStaticMetadataBuilder::dustbin(),0)); // null gets slot 0
        out
    }

    pub fn add(&mut self, request: AllotmentRequest) -> AllotmentHandle {
        if let Some(handle) = self.lookup(request.metadata().name()) {
            let mut data = self.allotments.lock().unwrap();
            let existing = data.data_mut().get_mut(&handle);
            existing.merge(&request);
            return handle;
        }
        self.allotments.lock().unwrap().add(request.metadata().name(),request.clone())
    }

    pub fn lookup(&mut self, name: &str) -> Option<AllotmentHandle> {
        self.allotments.lock().unwrap().get_handle(name).ok()
    }

    pub fn get(&self, handle: &AllotmentHandle) -> AllotmentRequest { self.allotments.lock().unwrap().data().get(handle).clone() }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum PositionVariant {
    HighPriority,
    LowPriority
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentPositionKind {
    Track,
    Overlay(i64),
    BaseLabel(PositionVariant),
    SpaceLabel(PositionVariant)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct OffsetSize(pub i64,pub(super) i64);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum AllotmentPosition {
    Track(OffsetSize),
    Overlay(i64),
    BaseLabel(PositionVariant,OffsetSize),
    SpaceLabel(PositionVariant,OffsetSize)
}

impl AllotmentPosition {
    pub fn kind(&self) -> AllotmentPositionKind {
        match self {
            AllotmentPosition::Track(_) => AllotmentPositionKind::Track,
            AllotmentPosition::Overlay(p) => AllotmentPositionKind::Overlay(*p),
            AllotmentPosition::BaseLabel(p,_) => AllotmentPositionKind::BaseLabel(p.clone()),
            AllotmentPosition::SpaceLabel(p,_) => AllotmentPositionKind::SpaceLabel(p.clone()),
        }
    }

    pub(super) fn update_metadata(&self, metadata: &AllotmentStaticMetadata) -> AllotmentStaticMetadata {
        let mut builder = AllotmentStaticMetadataBuilder::rebuild(metadata);
        match self {
            AllotmentPosition::Track(offset_size) => {
                builder.add_pair("type","track");
                builder.add_pair("offset",&offset_size.0.to_string());
                builder.add_pair("height",&offset_size.1.to_string());
            },
            _ => {
                builder.add_pair("type","other");
            }
        }
        AllotmentStaticMetadata::new(builder)
    }

    pub fn offset(&self) -> i64 { // XXX shouldn't exist. SHould magic shapes instead
        match self {
            AllotmentPosition::Track(x) => x.0,
            AllotmentPosition::BaseLabel(_,x) => x.0,
            AllotmentPosition::SpaceLabel(_,x) => x.0,
            AllotmentPosition::Overlay(x) => *x,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Allotment {
    position: AllotmentPosition,
    metadata: AllotmentStaticMetadata
}

impl Allotment {
    pub(super) fn new(position: AllotmentPosition, metadata: &AllotmentStaticMetadata) -> Allotment {
        Allotment { position, metadata: metadata.clone() }
    }

    pub fn position(&self) -> &AllotmentPosition { &self.position }
    pub fn metadata(&self) -> &AllotmentStaticMetadata { &self.metadata }
}
