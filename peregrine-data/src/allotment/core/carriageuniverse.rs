use std::{collections::HashMap, sync::{Arc}};
use peregrine_toolkit::{puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle}, log_extra};

use crate::{allotment::{style::{allotmentname::{AllotmentName, new_efficient_allotmentname_hashmap, BuildPassThroughHasher}, holder::ContainerHolder, stylebuilder::{make_transformable}, style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, util::bppxconverter::BpPxConverter}, ShapeRequestGroup, Shape, DataMessage, LeafRequest, CarriageShapeListRaw};

use super::{allotmentmetadata::{AllotmentMetadataReport, AllotmentMetadata, AllotmentMetadataBuilder}, aligner::Aligner, playingfield::PlayingField, leafrequest::LeafRequestMap, heighttracker::{HeightTrackerPieces, HeightTracker}};

pub(crate) struct CarriageUniversePrep {
    pub puzzle: PuzzleBuilder,
    pub metadata: AllotmentMetadataBuilder,
    pub root: Root,
    pub plm: LeafRequestMap,
    pub height_tracker: HeightTrackerPieces
}

impl CarriageUniversePrep {
    pub(crate) fn new(builder: &mut PuzzleBuilder) -> CarriageUniversePrep {
        CarriageUniversePrep {
            metadata: AllotmentMetadataBuilder::new(),
            root: Root::new(builder),
            plm: LeafRequestMap::new(),
            puzzle: builder.clone(),
            height_tracker: HeightTrackerPieces::new(&builder)
        }
    }
}

pub struct CarriageUniverseBuilder {
    leafs: HashMap<AllotmentName,LeafRequest,BuildPassThroughHasher>
}

impl CarriageUniverseBuilder {
    pub fn new() -> CarriageUniverseBuilder {
        CarriageUniverseBuilder {
            leafs: new_efficient_allotmentname_hashmap()
        }
    }

    pub fn pending_leaf(&mut self, spec: &str) -> &mut LeafRequest {
        let name = AllotmentName::new(spec);
        if !self.leafs.contains_key(&name) {
            self.leafs.insert(name.clone(),LeafRequest::new(&AllotmentName::new(spec)));
        }
        self.leafs.get_mut(&name).unwrap()
    }

    pub fn union(&self, other: &CarriageUniverseBuilder) -> CarriageUniverseBuilder {
        let mut leafs = self.leafs.clone();
        leafs.extend(&mut other.leafs.iter().map(|(k,v)| (k.clone(),v.clone())));
        CarriageUniverseBuilder {
            leafs
        }
    }

    fn make_transformable(&self, extent: Option<&ShapeRequestGroup>) -> Result<CarriageUniversePrep,DataMessage> {
        let mut builder = PuzzleBuilder::new();
        let mut prep = CarriageUniversePrep::new(&mut builder);
        let bp_px_converter = Arc::new(BpPxConverter::new(extent));
        make_transformable(&mut prep,&bp_px_converter,&mut self.leafs.values())?;
        Ok(prep)
    }
}

#[derive(Clone)]
pub struct CarriageUniverse {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    metadata: AllotmentMetadata,
    puzzle: Puzzle,
    root: Root,
    height_tracker: Arc<HeightTrackerPieces>
}

impl CarriageUniverse {
    pub fn new(builder: &CarriageUniverseBuilder, shapes: &[Shape<LeafRequest>], extent: Option<&ShapeRequestGroup>) -> Result<CarriageUniverse,DataMessage> {
        let prep = builder.make_transformable(extent)?;
        let shapes = shapes.iter().map(|x| 
            x.map_new_allotment(|x| x.transformable(&prep.plm).cloned())
        ).collect::<Vec<_>>();
        Ok(CarriageUniverse {
            shapes: Arc::new(shapes),
            metadata: AllotmentMetadata::new(&prep.metadata),
            puzzle: Puzzle::new(prep.puzzle.clone()),
            root: prep.root,
            height_tracker: Arc::new(prep.height_tracker)
        })
    }

    pub fn puzzle(&self) -> &Puzzle { &self.puzzle }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField {
        self.root.playing_field(solution)
    }

    pub fn get(&self, solution: &PuzzleSolution) -> Vec<Shape<LeafCommonStyle>> {
        /* */
        let heights = HeightTracker::new(&self.height_tracker,solution);
        #[cfg(debug_assertions)]
        log_extra!("heights {:?}",heights);
        /* */
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

#[derive(Clone)]
pub struct CarriageSolution {
    universe: CarriageUniverse,
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    solution: Arc<PuzzleSolution>
}

impl CarriageSolution {
    pub(crate) fn new(universe: &CarriageUniverse, height_tracker: &HeightTracker) -> CarriageSolution {
        let mut solution = PuzzleSolution::new(&universe.puzzle);
        universe.height_tracker.set_extra_height(&mut solution,height_tracker);
        solution.solve();
        let shapes = universe.get(&solution);
        CarriageSolution {
            universe: universe.clone(),
            shapes: Arc::new(shapes),
            solution: Arc::new(solution)
        }
    }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }

    pub fn playing_field(&self) -> PlayingField {
        self.universe.playing_field(&self.solution)
    }

    pub fn metadata(&self) -> AllotmentMetadataReport {
        self.universe.get_metadata(&self.solution)
    }
}
