use std::{hash::Hash, sync::Mutex};
use std::sync::Arc;
use peregrine_toolkit::lock;
use peregrine_toolkit::puzzle::PuzzleSolution;

use super::basicallotmentspec::BasicAllotmentSpec;
use super::rangeused::RangeUsed;
use crate::allotment::tree::allotmentbox::AllotmentBox;
use crate::{DataMessage, AllotmentMetadata, AllotmentMetadataRequest, CoordinateSystem, CoordinateSystemVariety};

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
    fn allotment(&self) -> Result<AllotmentBox,DataMessage>;
    fn up(self: Arc<Self>) -> Arc<dyn GenericAllotmentRequestImpl>;
    fn set_max_y(&self, max: i64);
    fn set_base_range(&self, used: &RangeUsed<f64>);
    fn set_pixel_range(&self, used: &RangeUsed<f64>);
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
    pub fn allotment(&self) -> Result<AllotmentBox,DataMessage> { self.0.allotment() }
    pub fn coord_system(&self) -> CoordinateSystem { self.0.coord_system() }
    pub fn set_base_range(&self, used: &RangeUsed<f64>) { self.0.set_base_range(used); }
    pub fn set_pixel_range(&self, used: &RangeUsed<f64>) { self.0.set_pixel_range(used); }
    pub fn set_max_y(&self, max: i64) { self.0.set_max_y(max); }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for AllotmentRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{{ AllotmentRequest name={} }}",self.name())
    }
}

pub struct AllotmentRequestExperience {
    allot_box: Option<Arc<AllotmentBox>>,
    base_range: RangeUsed<f64>,
    pixel_range: RangeUsed<f64>,
    max_y: i64
}

impl AllotmentRequestExperience {
    fn new() -> AllotmentRequestExperience {
        AllotmentRequestExperience {
            allot_box: None,
            base_range: RangeUsed::None,
            pixel_range: RangeUsed::None,
            max_y: 0
        }
    }

    fn set_transformer(&mut self, allot_box: Arc<AllotmentBox>) { 
        self.allot_box = Some(allot_box);
    }

    fn max_y(&self) -> i64 { self.max_y }
    fn set_max_y(&mut self, max: i64) { self.max_y = self.max_y.max(max); }

    fn base_range(&self) -> &RangeUsed<f64> { &self.base_range }
    fn set_base_range(&mut self, used: &RangeUsed<f64>) { self.base_range = self.base_range.merge(&used); }

    fn pixel_range(&self) -> &RangeUsed<f64> { &self.pixel_range }
    fn set_pixel_range(&mut self, used: &RangeUsed<f64>) { self.pixel_range = self.pixel_range.merge(&used); }

    fn add_allotment_metadata_values(&mut self, solution: &PuzzleSolution, metadata: &mut AllotmentMetadataRequest) {
        if let Some(allot_box) = &mut self.allot_box {
            allot_box.add_transform_metadata(solution, metadata);
        }
    }
}

pub struct AllotmentRequestImpl {
    metadata: AllotmentMetadata,
    name: String,
    priority: i64,
    experience: Mutex<AllotmentRequestExperience>,
    geometry: CoordinateSystem,
    depth: i8,
    ghost: bool
}

impl AllotmentRequestImpl {
    pub fn new(metadata: &AllotmentMetadata, geometry: &CoordinateSystem, depth: i8, ghost: bool) -> AllotmentRequestImpl {
        AllotmentRequestImpl {
            name: BasicAllotmentSpec::from_spec(metadata.name()).name().to_string(),
            priority: metadata.priority(),
            metadata: metadata.clone(),
            experience: Mutex::new(AllotmentRequestExperience::new()),
            depth, ghost,
            geometry: geometry.clone()
        }
    }

    pub fn set_allotment(&self, allot_box: Arc<AllotmentBox>) {
        if &self.name != "" {
            lock!(self.experience).set_transformer(allot_box);
        }
    }

    pub fn metadata(&self) -> &AllotmentMetadata { &self.metadata }
    pub fn max_y(&self) -> i64 { lock!(self.experience).max_y() }
    pub fn base_range(&self) -> RangeUsed<f64> { lock!(self.experience).base_range().clone() }
    pub fn pixel_range(&self) -> RangeUsed<f64> { lock!(self.experience).pixel_range().clone() }

    pub fn add_allotment_metadata_values(&self, solution: &PuzzleSolution, metadata: &mut AllotmentMetadataRequest) {
        lock!(self.experience).add_allotment_metadata_values(solution,metadata);
    }
}

impl GenericAllotmentRequestImpl for AllotmentRequestImpl {
    fn name(&self) -> &str { &self.name }
    fn is_dustbin(&self) -> bool { &self.name == "" }
    fn priority(&self) -> i64 { self.priority }
    fn depth(&self) -> i8 { self.depth }
    fn coord_system(&self) -> CoordinateSystem { self.geometry.clone() }

    fn allotment(&self) -> Result<AllotmentBox,DataMessage> {
        match lock!(self.experience).allot_box.clone() {
            Some(imp) => Ok(imp.as_ref().clone()),
            None => { return Err(DataMessage::AllotmentNotCreated(format!("name={}",self.metadata.name()))); }
        }
    }

    fn set_max_y(&self, max: i64) { lock!(self.experience).set_max_y(max); }
    fn set_base_range(&self, used: &RangeUsed<f64>) { lock!(self.experience).set_base_range(used); }
    fn set_pixel_range(&self, used: &RangeUsed<f64>) { lock!(self.experience).set_pixel_range(used); }

    fn up(self: Arc<Self>) -> Arc<dyn GenericAllotmentRequestImpl> { self }
}

impl AllotmentRequestImpl {
    pub fn new_dustbin() -> AllotmentRequestImpl {
        AllotmentRequestImpl {
            name: String::new(),
            priority: 0,
            metadata: AllotmentMetadata::new(AllotmentMetadataRequest::dustbin()),
            experience: Mutex::new(AllotmentRequestExperience::new()),
            depth: 0,
            ghost: true,
            geometry: CoordinateSystem(CoordinateSystemVariety::Dustbin,false)
        }
    }
}