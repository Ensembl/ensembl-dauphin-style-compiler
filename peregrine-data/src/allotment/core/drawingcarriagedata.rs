use std::sync::{Arc, Mutex};

use peregrine_toolkit::{lock, puzzle::{StaticAnswer}};

use crate::{Shape, LeafCommonStyle};

use super::{carriageoutput::CarriageOutput, trainstate::{CarriageTrainStateSpec, CarriageTrainStateRequest, TrainState3}};

/*
#[derive(Clone)]
pub struct DrawingCarriageData2 {
    shapes: Arc<Vec<Shape<LeafCommonStyle>>>,
}

impl DrawingCarriageData2 {
    pub(crate) fn new(universe: &CarriageOutput, train_state: &TrainState3) -> DrawingCarriageData2 {
        let answer_index = train_state.answer();
        let shapes = universe.get(&mut *lock!(answer_index));
        DrawingCarriageData2 {
            shapes: Arc::new(shapes)
        }
    }

    pub fn shapes(&self) -> &Arc<Vec<Shape<LeafCommonStyle>>> { &self.shapes }
}
*/