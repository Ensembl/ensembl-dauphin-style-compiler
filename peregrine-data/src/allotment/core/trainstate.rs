use std::sync::{Arc, Mutex};

use peregrine_toolkit::{puzzle::{StaticAnswer, AnswerAllocator}, lock, log};

use super::heighttracker::{HeightTracker, HeightTrackerPieces, HeightTrackerMerger, HeightTrackerPieces2, HeightTracker2Values};

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct TrainState {
    height_tracker: HeightTracker
}

impl TrainState {
    pub fn independent() -> TrainState {
        TrainState {
            height_tracker: HeightTracker::empty()
        }
    }

    pub(crate) fn new(height_tracker: HeightTracker) -> TrainState {
        TrainState {
            height_tracker
        }
    }

    pub(crate) fn update_puzzle(&self, answer_index: &mut StaticAnswer, height_tracker: &HeightTrackerPieces) {
        height_tracker.set_extra_height(answer_index,&self.height_tracker);
    }
}

/* Every carriage manipulates in a CarriageTrainStateRequest during creation (during build). This specifies the
 * requirements which a Carriage has of the train. 
 */

pub struct CarriageTrainStateRequest {
    height_tracker: HeightTrackerPieces2
}

impl CarriageTrainStateRequest {
    pub fn new() -> CarriageTrainStateRequest {
        CarriageTrainStateRequest {
            height_tracker: HeightTrackerPieces2::new()
        }
    }

    pub fn height_tracker(&self) -> &HeightTrackerPieces2 { &self.height_tracker }
    pub fn height_tracker_mut(&mut self) -> &mut HeightTrackerPieces2 { &mut self.height_tracker }
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct CarriageTrainStateSpec {
    height_values: HeightTracker2Values
}

impl CarriageTrainStateSpec {
    pub fn new(request: &CarriageTrainStateRequest, independent_answer: &StaticAnswer) -> CarriageTrainStateSpec {
        CarriageTrainStateSpec {
            height_values: HeightTracker2Values::new(request.height_tracker(),independent_answer)
        }
    }
}

#[derive(Clone,PartialEq,Eq,Hash)]
pub struct TrainState2 {
    height_tracker: HeightTracker
}

pub struct TrainStateBuilder {
//    carriages: HashMap<>,
    state: Arc<Mutex<TrainState2>>
}

impl TrainStateBuilder {
    pub fn new() -> TrainStateBuilder {
        TrainStateBuilder {
            state: Arc::new(Mutex::new(TrainState2{
                height_tracker: HeightTracker::empty()
            }))
        }
    }

    pub fn set_height_tracker(&self, height_tracker: HeightTracker) {
        lock!(self.state).height_tracker = height_tracker;
    }

    pub fn state_if_not(&self, was: Option<&TrainState2>) -> Option<TrainState2> {
        let state = lock!(self.state);
        if let Some(was) = was {
            if *state == *was { return None; }
        }
        Some(state.clone())
    }
}
