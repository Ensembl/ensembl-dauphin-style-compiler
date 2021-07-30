use std::{collections::HashMap, sync::{ Arc, Mutex }};
use std::hash::{ Hash };
use keyed::{ keyed_handle, KeyedValues, KeyedHandle };

#[derive(Debug)] // XXX Clone
pub struct AllotmentMetadata {
    name: String,
    pairs: HashMap<String,String>
}

impl AllotmentMetadata {
    pub fn dustbin() -> AllotmentMetadata {
        AllotmentMetadata::new("")
    }

    pub fn new(name: &str) -> AllotmentMetadata {
        AllotmentMetadata {
            name: name.to_string(),
            pairs: HashMap::new()
        }
    }

    pub fn add_pair(&mut self, key: &str, value: &str) {
        self.pairs.insert(key.to_string(),value.to_string());
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn is_dustbin(&self) -> bool { self.name == "" }

    pub fn kind(&self) -> AllotmentPositionKind { 
        if self.name.starts_with("window:") {
            AllotmentPositionKind::Overlay(if self.name.ends_with("-over") { 1 } else { 0 })
        } else {
            AllotmentPositionKind::Track
        }
    }
}

#[derive(Clone,Debug)]
pub struct AllotmentRequest {
    metadata: Arc<AllotmentMetadata>,
    priority: i64
}

impl AllotmentRequest {
    pub fn new(metadata: AllotmentMetadata, priority: i64) -> AllotmentRequest {
        AllotmentRequest {
            metadata: Arc::new(metadata),
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
    pub fn metadata(&self) -> &Arc<AllotmentMetadata> { &self.metadata }
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
        out.add(AllotmentRequest::new(AllotmentMetadata::dustbin(),0)); // null gets slot 0
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
    metadata: Arc<AllotmentMetadata>
}

impl Allotment {
    pub(super) fn new(position: AllotmentPosition, metadata: &Arc<AllotmentMetadata>) -> Allotment {
        Allotment { position, metadata: metadata.clone() }
    }

    pub fn position(&self) -> &AllotmentPosition { &self.position }
    pub fn metadata(&self) -> &Arc<AllotmentMetadata> { &self.metadata }
}
