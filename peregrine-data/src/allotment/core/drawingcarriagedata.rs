use std::sync::{Arc, Mutex};

use peregrine_toolkit::{lock, puzzle::{StaticAnswer}};

use crate::{Shape, LeafCommonStyle};

use super::{carriageoutput::CarriageOutput, trainstate::{CarriageTrainStateSpec, CarriageTrainStateRequest, TrainState3}};

#[derive(Clone)]
pub struct DrawingCarriageData2 {
    universe: CarriageOutput,
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
    answer_index: Arc<Mutex<StaticAnswer>>
}

impl DrawingCarriageData2 {
    pub(crate) fn new(universe: &CarriageOutput, train_state: &TrainState3) -> DrawingCarriageData2 {
        let mut answer_index = train_state.answer();
        let shapes = universe.get(&mut *lock!(answer_index));
        DrawingCarriageData2 {
            universe: universe.clone(),
            shapes: Arc::new(shapes),
            answer_index
        }
    }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }
}
