use crate::AllotmentRequestBuilder;
use crate::AllotmentRequest;
use crate::Track;
use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::Hasher, sync::{ Arc, Mutex }};
use std::hash::{ Hash };
use keyed::{ keyed_handle, KeyedHandle };
use super::pitch::Pitch;

#[derive(Clone,Debug)]
pub struct AllotterMetadata {
    allotments: Arc<Vec<AllotmentRequest>>,
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
    pub fn new(allotments: Vec<AllotmentRequest>) -> AllotterMetadata {
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

#[derive(Clone)]
pub struct AllotmentPetitioner {
    allotments: Arc<Mutex<HashMap<String,AllotmentRequest>>>,
}

impl AllotmentPetitioner {
    pub fn new() -> AllotmentPetitioner {
        let mut out = AllotmentPetitioner {
            allotments: Arc::new(Mutex::new(HashMap::new()))
        };
        out.add(AllotmentRequestBuilder::dustbin()); // null gets slot 0
        out
    }

    pub fn add(&mut self, builder: AllotmentRequestBuilder) -> AllotmentRequest {
        let request = AllotmentRequest::new(builder);
        let mut allotments = self.allotments.lock().unwrap();
        if let Some(request) = allotments.get(request.name()) {
            return request.clone();
        }
        allotments.insert(request.name().to_string(),request.clone());
        return request
    }

    pub fn lookup(&mut self, name: &str) -> Option<AllotmentRequest> {
        self.allotments.lock().unwrap().get(name).cloned()
    }
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

impl AllotmentPositionKind {
    pub(crate) fn base_filter(&self) -> bool {
        match self {
            AllotmentPositionKind::Track => true,
            AllotmentPositionKind::Overlay(_) => false,
            AllotmentPositionKind::BaseLabel(_) => true,
            AllotmentPositionKind::SpaceLabel(_) => false
        }
    }
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

    pub(super) fn update_metadata(&self, metadata: &AllotmentRequest) -> AllotmentRequest {
        let mut builder = AllotmentRequestBuilder::rebuild(metadata);
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
        AllotmentRequest::new(builder)
    }

    pub fn offset(&self) -> i64 { // XXX shouldn't exist. SHould magic shapes instead
        match self {
            AllotmentPosition::Track(x) => x.0,
            AllotmentPosition::BaseLabel(_,x) => x.0,
            AllotmentPosition::SpaceLabel(_,x) => x.0,
            AllotmentPosition::Overlay(x) => *x,
        }
    }

    pub fn apply_pitch(&self, pitch: &mut Pitch) {
        match self {
            AllotmentPosition::Track(offset_size) => {
                pitch.set_limit(offset_size.0+offset_size.1);
            },
            _ => {}
        }        
    }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Allotment {
    position: AllotmentPosition,
    metadata: AllotmentRequest
}

impl Allotment {
    pub(super) fn new(position: AllotmentPosition, metadata: &AllotmentRequest) -> Allotment {
        Allotment { position, metadata: metadata.clone() }
    }

    pub fn position(&self) -> &AllotmentPosition { &self.position }
    pub fn metadata(&self) -> &AllotmentRequest { &self.metadata }
}
