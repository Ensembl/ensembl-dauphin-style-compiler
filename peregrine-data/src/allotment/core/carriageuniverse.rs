use std::{collections::HashMap, sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{PuzzleBuilder, PuzzleSolution, Puzzle}, lock, log};

use crate::{allotment::{style::{allotmentname::{AllotmentName, new_efficient_allotmentname_hashmap, BuildPassThroughHasher}, stylebuilder::{make_transformable}, style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, util::bppxconverter::BpPxConverter, collision::bumperfactory::BumperFactory}, ShapeRequestGroup, Shape, DataMessage, LeafRequest};

use super::{allotmentmetadata::{AllotmentMetadataReport, AllotmentMetadata, AllotmentMetadataBuilder}, playingfield::PlayingField, leafrequest::LeafRequestMap, heighttracker::{HeightTrackerPieces, HeightTracker}, trainstate::TrainState};

pub(crate) struct CarriageUniversePrep {
    pub puzzle: PuzzleBuilder,
    pub metadata: AllotmentMetadataBuilder,
    pub root: Root,
    pub plm: LeafRequestMap,
    pub height_tracker: HeightTrackerPieces,
    pub bumper_factory: BumperFactory
}

impl CarriageUniversePrep {
    pub(crate) fn new(builder: &mut PuzzleBuilder) -> CarriageUniversePrep {
        CarriageUniversePrep {
            metadata: AllotmentMetadataBuilder::new(),
            root: Root::new(builder),
            plm: LeafRequestMap::new(),
            puzzle: builder.clone(),
            height_tracker: HeightTrackerPieces::new(&builder),
            bumper_factory: BumperFactory::new()
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

    fn height_tracker(&self, solution: &PuzzleSolution) -> HeightTracker {
        HeightTracker::new(&self.height_tracker,solution)
    }

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

#[derive(Clone)]
pub struct CarriageSolution {
    universe: CarriageUniverse,
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    solution: Arc<PuzzleSolution>
}

impl CarriageSolution {
    pub(crate) fn new(universe: &CarriageUniverse, train_state: &TrainState) -> CarriageSolution {
        let mut solution = PuzzleSolution::new(&universe.puzzle);
        train_state.update_puzzle(&mut solution,&universe.height_tracker);
        solution.solve();
        let shapes = universe.get(&solution);
        CarriageSolution {
            universe: universe.clone(),
            shapes: Arc::new(shapes),
            solution: Arc::new(solution)
        }
    }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }
    pub fn height_tracker(&self) -> HeightTracker { self.universe.height_tracker(&self.solution) }

    pub fn playing_field(&self) -> PlayingField {
        self.universe.playing_field(&self.solution)
    }

    pub fn metadata(&self) -> AllotmentMetadataReport {
        self.universe.get_metadata(&self.solution)
    }
}

#[derive(Clone)]
struct CarriageShapeCache {
    train_state: TrainState,
    shapes: Option<CarriageSolution>
}

impl CarriageShapeCache {
    fn new(train_state: &TrainState) -> CarriageShapeCache {
        CarriageShapeCache {
            train_state: train_state.clone(),
            shapes: None
        }
    }

    fn get(&mut self, universe: &CarriageUniverse, train_state: &TrainState) -> Option<&CarriageSolution> {
        if train_state != &self.train_state { return None; }
        if self.shapes.is_none() {
            log!("new solution!");
            self.shapes = Some(CarriageSolution::new(universe,&self.train_state));
        }
        Some(self.shapes.as_ref().unwrap())
    }

    fn set(&mut self, train_state: &TrainState, shapes: &CarriageSolution) {
        self.train_state = train_state.clone();
        self.shapes = Some(shapes.clone());
    }
}

#[derive(Clone)]
pub struct CarriageShapes {
    universe: CarriageUniverse,
    independent: Arc<Mutex<CarriageShapeCache>>,
    cache: Arc<Mutex<CarriageShapeCache>>
}

impl CarriageShapes {
    pub fn new(universe: &CarriageUniverse) -> CarriageShapes {
        let independent_state = TrainState::independent();
        CarriageShapes {
            universe: universe.clone(),
            cache: Arc::new(Mutex::new(CarriageShapeCache::new(&independent_state))),
            independent: Arc::new(Mutex::new(CarriageShapeCache::new(&independent_state)))
        }
    }

    fn try_get(&self, whither: &Arc<Mutex<CarriageShapeCache>>, train_state: &TrainState) -> Option<CarriageSolution> {
        lock!(whither).get(&self.universe,train_state).cloned()
    }

    pub(crate) fn get(&self, state: &TrainState) -> CarriageSolution {
        if let Some(solution) = self.try_get(&self.independent,state) {
            return solution.clone();
        }
        if let Some(solution) = self.try_get(&self.cache,state) {
            return solution.clone();
        }
        let shapes = CarriageSolution::new(&self.universe,state);
        lock!(self.cache).set(state,&shapes);
        shapes
    }
}
