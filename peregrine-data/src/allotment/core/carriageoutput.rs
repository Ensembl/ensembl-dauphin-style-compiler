use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{Answer, AnswerAllocator, StaticAnswer}, lock};

use crate::{allotment::{style::{style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, collision::{bumperfactory::BumperFactory}, util::bppxconverter::BpPxConverter}, ShapeRequestGroup, Shape, DataMessage, LeafRequest};

use super::{allotmentmetadata::{AllotmentMetadataReport, AllotmentMetadata, AllotmentMetadataBuilder}, playingfield::PlayingField, leafrequest::LeafTransformableMap, heighttracker::{HeightTrackerPieces}, leaflist::LeafList, trainstate::{CarriageTrainStateRequest, CarriageTrainStateSpec}};

pub(crate) struct BoxPositionContext {
    //pub bump_requests: BumpRequests,
    pub independent_answer: StaticAnswer,
    pub bp_px_converter: Arc<BpPxConverter>,
    pub metadata: AllotmentMetadataBuilder,
    pub root: Root,
    pub plm: LeafTransformableMap,
    pub height_tracker: HeightTrackerPieces,
    pub state_request: CarriageTrainStateRequest,
    pub bumper_factory: BumperFactory
}

impl BoxPositionContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>, answer_allocator: &Arc<Mutex<AnswerAllocator>>) -> BoxPositionContext {
        //let region = extent.map(|x| x.region().clone());
        let independent_answer = lock!(answer_allocator).get();
        BoxPositionContext {
            bp_px_converter: Arc::new(BpPxConverter::new(extent)),
            metadata: AllotmentMetadataBuilder::new(),
            //bump_requests: BumpRequests::new(region.as_ref().map(|r| r.index()).unwrap_or(0) as usize),
            root: Root::new(),
            plm: LeafTransformableMap::new(),
            height_tracker: HeightTrackerPieces::new(),
            independent_answer,
            state_request: CarriageTrainStateRequest::new(),
            bumper_factory: BumperFactory::new()
        }
    }
}

#[derive(Clone)]
pub struct CarriageOutput {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    spec: Arc<CarriageTrainStateSpec>,
    metadata: AllotmentMetadata,
    answer_index_allocator: Arc<Mutex<AnswerAllocator>>,
    root: Root,
    height_tracker: Arc<HeightTrackerPieces>
}

impl CarriageOutput {
    pub fn new(builder: &LeafList, shapes: &[Shape<LeafRequest>], extent: Option<&ShapeRequestGroup>, answer_allocator: &Arc<Mutex<AnswerAllocator>>) -> Result<CarriageOutput,DataMessage> {
        let (prep,spec) = builder.position_boxes(extent,answer_allocator)?;
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|r| prep.plm.transformable(r.name()).cloned())
        ).collect::<Vec<_>>();
        Ok(CarriageOutput {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata::new(&prep.metadata),
            answer_index_allocator: Arc::new(Mutex::new(AnswerAllocator::new())),
            root: prep.root,
            spec: Arc::new(spec),
            height_tracker: Arc::new(prep.height_tracker)
        })
    }

    pub fn make_answer_index<'a>(&self) -> Answer<'a> {
        let mut aia = lock!(self.answer_index_allocator);
        aia.get()
    }

    pub(super) fn height_tracker_pieces(&self) -> &HeightTrackerPieces { &self.height_tracker }

    pub fn playing_field(&self, answer_index: &mut StaticAnswer) -> PlayingField {
        self.root.playing_field(answer_index)
    }

    pub fn get(&self, answer_index: &mut StaticAnswer) -> Vec<Shape<LeafCommonStyle>> {
        let mut out = vec![];
        for input in self.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(answer_index)).make());
        }
        out
    }

    pub fn get_metadata(&self, answer_index: &mut StaticAnswer) -> AllotmentMetadataReport {
        self.metadata.get(answer_index)
    }
}
