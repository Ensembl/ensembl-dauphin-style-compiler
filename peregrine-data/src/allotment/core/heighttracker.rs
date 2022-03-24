use std::{collections::HashMap, sync::Arc, fmt, hash::Hash};

use peregrine_toolkit::puzzle::{PuzzleValueHolder, PuzzleBuilder, FoldValue, PuzzleSolution, ClonablePuzzleValue, PuzzlePiece};

use crate::allotment::{style::allotmentname::AllotmentName};

struct HeightTrackerEntry {
    extra: PuzzlePiece<f64>,
    used: FoldValue<f64>
}

impl HeightTrackerEntry {
    fn add(&mut self, height: &PuzzleValueHolder<f64>) {
        self.used.add(height);
    }

    fn build(&mut self) {
        self.used.build();
    }

    fn get_used(&self, solution: &PuzzleSolution) -> f64 {
        self.used.get().get_clone(solution)
    }

    fn set_full_height(&self, solution: &mut PuzzleSolution, full_height: f64) {
        self.extra.set_answer(solution,full_height);
    }
}

pub struct HeightTrackerPieces {
    puzzle: PuzzleBuilder,
    heights: HashMap<AllotmentName,HeightTrackerEntry>
}

impl HeightTrackerPieces {
    pub(crate) fn new(puzzle: &PuzzleBuilder) -> HeightTrackerPieces {
        HeightTrackerPieces {
            puzzle: puzzle.clone(),
            heights: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, name: &AllotmentName, height: &PuzzleValueHolder<f64>) {
        self.heights.entry(name.clone()).or_insert({
            let mut output = self.puzzle.new_piece();
            #[cfg(debug_assertions)]
            output.set_name("height tracker in");
            let mut extra = self.puzzle.new_piece_default(0.); // XXX no default
            #[cfg(debug_assertions)]
            extra.set_name("height tracker out");
            HeightTrackerEntry {  
                extra,
                used: FoldValue::new(output,|a,b| { f64::max(a,b) })
            }
        }).add(height);
    }

    pub(crate) fn build(&mut self) {
        for values in self.heights.values_mut() {
            values.build();
        }
    }

    /* must called pre-solve */
    pub(crate) fn set_extra_height(&self, solution: &mut PuzzleSolution, heights: &HeightTracker) {
        for (name,entry) in &self.heights {
            entry.set_full_height(solution,heights.get(name));
        }
    }
}

pub struct HeightTracker {
    heights: HashMap<AllotmentName,f64>
}

impl HeightTracker {
    pub(crate) fn empty() -> HeightTracker {
        HeightTracker {
            heights: HashMap::new()
        }
    }

    pub(crate) fn new(pieces: &HeightTrackerPieces, solution: &PuzzleSolution) -> HeightTracker {
        let mut out = HashMap::new();
        for (name,height) in pieces.heights.iter() {
            out.insert(name.clone(),height.get_used(solution));
        }
        HeightTracker { heights: out }
    }

    fn get(&self, name: &AllotmentName) -> f64 {
        self.heights.get(name).cloned().unwrap_or(0.)
    }
}

#[cfg(debug_assertions)]
impl fmt::Debug for HeightTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = vec![];
        for (name,height) in &self.heights {
            out.push(format!("{:?}: {}",name,height));
        }
        write!(f,"heights: {}",out.join(", "))
    }
}
