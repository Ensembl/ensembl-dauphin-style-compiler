use std::{hash::Hash, sync::Mutex};
use std::sync::Arc;
use peregrine_toolkit::lock;

use super::basicallotmentspec::BasicAllotmentSpec;
use crate::allotment::tree::leaftransformer::LeafGeometry;
use crate::{Allotment, DataMessage, AllotmentMetadata, AllotmentMetadataRequest};

use super::allotment::Transformer;
use super::{allotment::CoordinateSystem, dustbinallotment::DustbinAllotment};

impl Hash for AllotmentRequest {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.name().hash(state);
    }
}

impl PartialEq for AllotmentRequest {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0.name().clone();
        let b = other.0.name().clone();
        a == b
    }
}

impl Eq for AllotmentRequest {}

pub trait GenericAllotmentRequestImpl {
    fn name(&self) -> &str;
    fn is_dustbin(&self) -> bool;
    fn priority(&self) -> i64;
    fn allotment(&self) -> Result<Allotment,DataMessage>;
    fn up(self: Arc<Self>) -> Arc<dyn GenericAllotmentRequestImpl>;
    fn set_max_y(&self, max: i64);
    fn set_base_range(&self, used: &RangeUsed);
    fn coord_system(&self) -> CoordinateSystem;
    fn depth(&self) -> i8;
}

#[derive(Clone)]
pub struct AllotmentRequest(Arc<dyn GenericAllotmentRequestImpl>);

impl AllotmentRequest {
    pub(crate) fn upcast<T>(request: Arc<T>) -> AllotmentRequest where T: GenericAllotmentRequestImpl + 'static + ?Sized {
        AllotmentRequest(request.up())
    }

    pub fn name(&self) -> String { self.0.name().to_string() }
    pub fn is_dustbin(&self) -> bool { self.0.is_dustbin() }
    pub fn priority(&self) -> i64 { self.0.priority() }
    pub fn depth(&self) -> i8 { self.0.depth() }
    pub fn allotment(&self) -> Result<Allotment,DataMessage> { self.0.allotment() }
    pub fn coord_system(&self) -> CoordinateSystem { self.0.coord_system() }
    pub fn set_base_range(&self, used: &RangeUsed) { self.0.set_base_range(used); }
    pub fn set_max_y(&self, max: i64) { self.0.set_max_y(max); }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone)]
pub enum RangeUsed {
    None,
    All,
    Part(f64,f64)
}

impl RangeUsed {
    fn merge(&self, other: &RangeUsed) -> RangeUsed {
        match (self,other) {
            (RangeUsed::All,_) => RangeUsed::All,
            (_,RangeUsed::All) => RangeUsed::All,
            (RangeUsed::None,x) => x.clone(),
            (x,RangeUsed::None) => x.clone(),
            (RangeUsed::Part(a1,b1), RangeUsed::Part(a2,b2)) => {
                let (a1,b1) = if a1<b1 { (a1,b1) } else { (b1,a1) };
                let (a2,b2) = if a2<b2 { (a2,b2) } else { (b2,a2) };
                RangeUsed::Part(a1.min(*a2),b1.max(*b2))
            }
        }
    }
}

pub struct AllotmentRequestImpl<T: Transformer> {
    metadata: AllotmentMetadata,
    name: String,
    priority: i64,
    transformer: Mutex<Option<Arc<T>>>,
    geometry: LeafGeometry,
    depth: i8,
    max: Mutex<i64>,
    base_range: Mutex<RangeUsed>,
    ghost: bool
}

impl<T: Transformer> AllotmentRequestImpl<T> {
    pub fn new(metadata: &AllotmentMetadata, geometry: &LeafGeometry, depth: i8, ghost: bool) -> AllotmentRequestImpl<T> {
        AllotmentRequestImpl {
            name: BasicAllotmentSpec::from_spec(metadata.name()).name().to_string(),
            priority: metadata.priority(),
            metadata: metadata.clone(),
            transformer: Mutex::new(None),
            depth, ghost,
            geometry: geometry.clone(),
            base_range: Mutex::new(RangeUsed::None),
            max: Mutex::new(0)
        }
    }

    pub fn set_allotment(&self, value: Arc<T>) {
        if &self.name != "" {
            *self.transformer.lock().unwrap() = Some(value);
        }
    }

    pub fn ghost(&self) -> bool { self.ghost }
    pub fn geometry(&self) -> &LeafGeometry { &self.geometry }
    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }
    pub fn max_y(&self) -> i64 { *self.max.lock().unwrap() }
    pub fn base_range(&self) -> RangeUsed { lock!(self.base_range).clone() }

    pub fn transformer(&self) -> Option<Arc<T>> {
        self.transformer.lock().unwrap().as_ref().cloned()
    }

    pub fn add_allotment_metadata_values(&self, metadata: &mut AllotmentMetadataRequest) {
        if let Some(transformer) = lock!(self.transformer).as_ref() {
            transformer.add_transform_metadata(metadata);
        }
    }
}

impl<T: Transformer + 'static> GenericAllotmentRequestImpl for AllotmentRequestImpl<T> {
    fn name(&self) -> &str { &self.name }
    fn is_dustbin(&self) -> bool { &self.name == "" }
    fn priority(&self) -> i64 { self.priority }
    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.geometry.coord_system().clone() }

    fn allotment(&self) -> Result<Allotment,DataMessage> {
        match self.transformer.lock().unwrap().clone() {
            Some(imp) => Ok(Allotment::new(imp)),
            None => Err(DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name())))
        }
    }

    fn set_max_y(&self, max: i64) {
        let mut self_max = lock!(self.max);
        *self_max = (*self_max).max(max);
    }

    fn set_base_range(&self, used: &RangeUsed) {
        let mut range = lock!(self.base_range);
        *range = range.merge(&used);
    }

    fn up(self: Arc<Self>) -> Arc<dyn GenericAllotmentRequestImpl> { self }
}

impl AllotmentRequestImpl<DustbinAllotment> {
    pub fn new_dustbin() -> AllotmentRequestImpl<DustbinAllotment> {
        AllotmentRequestImpl {
            name: String::new(),
            priority: 0,
            metadata: AllotmentMetadata::new(AllotmentMetadataRequest::dustbin()),
            transformer: Mutex::new(None),
            depth: 0,
            ghost: true,
            geometry: LeafGeometry::new(CoordinateSystem::Window,false),
            max: Mutex::new(0),
            base_range: Mutex::new(RangeUsed::None)
        }
    }
}
