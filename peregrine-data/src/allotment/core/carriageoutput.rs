use std::{sync::{Arc}};
use peregrine_toolkit::{puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle}};

use crate::{allotment::{style::{style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, collision::{bumperfactory::BumperFactory, bumpprocess::{BumpRequests}}}, ShapeRequestGroup, Shape, DataMessage, LeafRequest, Region};

use super::{allotmentmetadata::{AllotmentMetadataReport, AllotmentMetadata, AllotmentMetadataBuilder}, playingfield::PlayingField, leafrequest::LeafTransformableMap, heighttracker::{HeightTrackerPieces}, leaflist::LeafList};

pub(crate) struct CarriageUniversePrep {
    pub puzzle: PuzzleBuilder,
    pub bump_requests: BumpRequests,
    pub metadata: AllotmentMetadataBuilder,
    pub root: Root,
    pub plm: LeafTransformableMap,
    pub height_tracker: HeightTrackerPieces,
    pub bumper_factory: BumperFactory
}

impl CarriageUniversePrep {
    pub(crate) fn new(builder: &mut PuzzleBuilder, region: Option<Region>) -> CarriageUniversePrep {
        CarriageUniversePrep {
            metadata: AllotmentMetadataBuilder::new(),
            bump_requests: BumpRequests::new(region.as_ref().map(|r| r.index()).unwrap_or(0) as usize),
            root: Root::new(builder),
            plm: LeafTransformableMap::new(),
            puzzle: builder.clone(),
            height_tracker: HeightTrackerPieces::new(&builder),
            bumper_factory: BumperFactory::new()
        }
    }
}

#[derive(Clone)]
pub struct CarriageOutput {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    metadata: AllotmentMetadata,
    puzzle: Puzzle,
    root: Root,
    height_tracker: Arc<HeightTrackerPieces>
}

impl CarriageOutput {
    pub fn new(builder: &LeafList, shapes: &[Shape<LeafRequest>], extent: Option<&ShapeRequestGroup>) -> Result<CarriageOutput,DataMessage> {
        let prep = builder.make_transformable(extent)?;
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|r| prep.plm.transformable(r.name()).cloned())
        ).collect::<Vec<_>>();
        Ok(CarriageOutput {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata::new(&prep.metadata),
            puzzle: Puzzle::new(prep.puzzle.clone()),
            root: prep.root,
            height_tracker: Arc::new(prep.height_tracker)
        })
    }

    pub(super) fn height_tracker_pieces(&self) -> &HeightTrackerPieces { &self.height_tracker }

    pub fn puzzle(&self) -> &Puzzle { &self.puzzle }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField {
        self.root.playing_field(solution)
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> {
        let mut out = vec![];
        for input in self.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(solution)).make(solution));
        }
        out
    }

    pub fn get_metadata(&self, solution: &PuzzleSolution) -> AllotmentMetadataReport {
        self.metadata.get(solution)
    }
}
