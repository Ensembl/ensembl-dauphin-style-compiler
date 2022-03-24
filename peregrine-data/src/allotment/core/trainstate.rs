use peregrine_toolkit::puzzle::PuzzleSolution;

use super::heighttracker::{HeightTracker, HeightTrackerPieces};

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

    pub(crate) fn update_puzzle(&self, solution: &mut PuzzleSolution, height_tracker: &HeightTrackerPieces) {
        height_tracker.set_extra_height(solution,&self.height_tracker);
    }
}
