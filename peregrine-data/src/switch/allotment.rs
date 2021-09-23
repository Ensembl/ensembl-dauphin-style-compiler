use crate::AllotmentRequest;
use crate::SpaceBasePointRef;
use crate::spacebase::spacebase::SpaceBasePoint;
use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::Hasher, sync::{ Arc, Mutex }};
use std::hash::{ Hash };
use super::allotmentrequest::AllotmentMetadata;
use super::pitch::Pitch;
use peregrine_toolkit::lock;

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
pub struct AllAllotmentsRequest {
    allotments: Arc<Mutex<HashMap<String,AllotmentRequest>>>,
}

impl AllAllotmentsRequest {
    pub fn new() -> AllAllotmentsRequest {
        let mut out = AllAllotmentsRequest {
            allotments: Arcuse_allotment::new(Mutex::new(HashMap::new()))
        };
        out.add(AllotmentMetadata::dustbin()); // null gets slot 0
        out
    }

    pub fn add(&mut self, metadata: AllotmentMetadata) {
        let request = AllotmentRequest::new(metadata);
        let mut allotments = self.allotments.lock().unwrap();
        if allotments.get(request.name()).is_some() {
            return;
        }
        allotments.insert(request.name().to_string(),request.clone());
    }

    pub fn lookup(&mut self, name: &str) -> Option<AllotmentRequest> {
        self.allotments.lock().unwrap().get(name).cloned()
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentDirection {
    Forward,
    Reverse
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum AllotmentGroup {
    Track,
    Overlay(i64),
    BaseLabel(AllotmentDirection),
    SpaceLabel(AllotmentDirection)
}

impl AllotmentGroup {
    pub(crate) fn base_filter(&self) -> bool {
        match self {
            AllotmentGroup::Track => true,
            AllotmentGroup::Overlay(_) => false,
            AllotmentGroup::BaseLabel(_) => true,
            AllotmentGroup::SpaceLabel(_) => false
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
    BaseLabel(AllotmentDirection,OffsetSize),
    SpaceLabel(AllotmentDirection,OffsetSize)
}

impl AllotmentPosition {
    pub fn allotment_group(&self) -> AllotmentGroup {
        match self {
            AllotmentPosition::Track(_) => AllotmentGroup::Track,
            AllotmentPosition::Overlay(p) => AllotmentGroup::Overlay(*p),
            AllotmentPosition::BaseLabel(p,_) => AllotmentGroup::BaseLabel(p.clone()),
            AllotmentPosition::SpaceLabel(p,_) => AllotmentGroup::SpaceLabel(p.clone()),
        }
    }

    pub(super) fn update_metadata(&self, metadata: &AllotmentRequest) -> AllotmentRequest {
        let mut builder = AllotmentMetadata::rebuild(metadata);
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

pub trait AllotmentImpl {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64>;
    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>>;
    fn direction(&self) -> AllotmentDirection;    
    fn apply_pitch(&self, pitch: &mut Pitch);
    fn metadata(&self) -> &AllotmentRequest;
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct GeneralAllotment {
    position: AllotmentPosition,
    metadata: AllotmentRequest
}

impl GeneralAllotment {
    pub(super) fn new(position: AllotmentPosition, metadata: &AllotmentRequest) -> GeneralAllotment {
        GeneralAllotment { position, metadata: metadata.clone() }
    }
}

impl AllotmentImpl for GeneralAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        let mut output = input.make();
        output.normal += self.position.offset() as f64;
        output
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        let offset = self.position.offset() as f64;
        values.iter().map(|x| x.map(|y| y+offset)).collect()
    }

    fn direction(&self) -> AllotmentDirection {
        match &self.position {
            AllotmentPosition::BaseLabel(p,_) => p.clone(),
            AllotmentPosition::SpaceLabel(p,_) => p.clone(),
            _ => AllotmentDirection::Forward
        }
    }
    
    fn apply_pitch(&self, pitch: &mut Pitch) {
        self.position.apply_pitch(pitch);
    }

    fn metadata(&self) -> &AllotmentRequest { &self.metadata }
}

pub struct AllAllotment {
    metadata: AllotmentRequest
}

impl AllotmentImpl for AllAllotment {
    fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        input.make() // XXX
    }

    fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        values.to_vec() // XXX
    }

    fn direction(&self) -> AllotmentDirection {
        AllotmentDirection::Forward
    }

    fn apply_pitch(&self, pitch: &mut Pitch) {}

    fn metadata(&self) -> &AllotmentRequest { &self.metadata }
}

#[derive(Clone)]
pub struct Allotment(Arc<Mutex<dyn AllotmentImpl>>);

impl Allotment {
    pub fn new(position: AllotmentPosition, metadata: &AllotmentRequest) -> Allotment { // XXX
        Allotment(Arc::new(Mutex::new(GeneralAllotment::new(position,metadata))))
    }

    pub fn transform_spacebase(&self, input: &SpaceBasePointRef<f64>) -> SpaceBasePoint<f64> {
        lock!(self.0).transform_spacebase(input)
    }

    pub fn transform_yy(&self, values: &[Option<f64>]) -> Vec<Option<f64>> {
        lock!(self.0).transform_yy(values)
    }

    pub fn direction(&self) -> AllotmentDirection {
        lock!(self.0).direction().clone()
    }

    pub fn apply_pitch(&self, pitch: &mut Pitch) {
        lock!(self.0).apply_pitch(pitch)
    }

    pub fn metadata(&self) -> AllotmentRequest {
        lock!(self.0).metadata().clone()
    }
}
