use std::sync::{Arc, Mutex};

use peregrine_toolkit::{lock, puzzle::{StaticAnswer}};

use crate::{Shape, LeafCommonStyle, TrainState, PlayingField, AllotmentMetadataReport};

use super::{carriageoutput::CarriageOutput, heighttracker::HeightTracker};

#[derive(Clone)]
pub struct DrawingCarriageData {
    universe: CarriageOutput,
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    answer_index: Arc<Mutex<StaticAnswer>>
}

impl DrawingCarriageData {
    pub(crate) fn new(universe: &CarriageOutput, train_state: &TrainState) -> DrawingCarriageData {
        let mut answer_index = universe.make_answer_index();
        train_state.update_puzzle(&mut answer_index,universe.height_tracker_pieces());
        let shapes = universe.get(&mut answer_index);
        DrawingCarriageData {
            universe: universe.clone(),
            shapes: Arc::new(shapes),
            answer_index: Arc::new(Mutex::new(answer_index))
        }
    }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }

    pub fn height_tracker(&self) -> HeightTracker {    
        let mut answer_index = lock!(self.answer_index);
        HeightTracker::new(&self.universe.height_tracker_pieces(),&mut answer_index)
    }

    pub fn playing_field(&self) -> PlayingField {
        let mut answer_index = lock!(self.answer_index);
        self.universe.playing_field(&mut answer_index)
    }

    pub fn metadata(&self) -> AllotmentMetadataReport {
        let mut answer_index = lock!(self.answer_index);
        self.universe.get_metadata(&mut answer_index)
    }
}

#[derive(Clone)]
struct CarriageShapeCache {
    train_state: TrainState,
    shapes: Option<DrawingCarriageData>
}

impl CarriageShapeCache {
    fn new(train_state: &TrainState) -> CarriageShapeCache {
        CarriageShapeCache {
            train_state: train_state.clone(),
            shapes: None
        }
    }

    fn get(&mut self, universe: &CarriageOutput, train_state: &TrainState) -> Option<&DrawingCarriageData> {
        if train_state != &self.train_state { return None; }
        if self.shapes.is_none() {
            self.shapes = Some(DrawingCarriageData::new(universe,&self.train_state));
        }
        Some(self.shapes.as_ref().unwrap())
    }

    fn set(&mut self, train_state: &TrainState, shapes: &DrawingCarriageData) {
        self.train_state = train_state.clone();
        self.shapes = Some(shapes.clone());
    }
}

#[derive(Clone)]
pub struct DrawingCarriageDataStore {
    universe: CarriageOutput,
    independent: Arc<Mutex<CarriageShapeCache>>,
    cache: Arc<Mutex<CarriageShapeCache>>
}

impl DrawingCarriageDataStore {
    pub fn new(universe: &CarriageOutput) -> DrawingCarriageDataStore {
        let independent_state = TrainState::independent();
        DrawingCarriageDataStore {
            universe: universe.clone(),
            cache: Arc::new(Mutex::new(CarriageShapeCache::new(&independent_state))),
            independent: Arc::new(Mutex::new(CarriageShapeCache::new(&independent_state)))
        }
    }

    fn try_get(&self, whither: &Arc<Mutex<CarriageShapeCache>>, train_state: &TrainState) -> Option<DrawingCarriageData> {
        lock!(whither).get(&self.universe,train_state).cloned()
    }

    pub(crate) fn get(&self, state: &TrainState) -> DrawingCarriageData {
        if let Some(solution) = self.try_get(&self.independent,state) {
            return solution.clone();
        }
        if let Some(solution) = self.try_get(&self.cache,state) {
            return solution.clone();
        }
        let shapes = DrawingCarriageData::new(&self.universe,state);
        lock!(self.cache).set(state,&shapes);
        shapes
    }
}
