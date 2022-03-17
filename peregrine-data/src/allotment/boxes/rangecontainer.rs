use std::sync::{Arc, Mutex};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleBuilder, PuzzleValue, ClonablePuzzleValue, ConstantPuzzlePiece}, lock, log};

use crate::allotment::core::rangeused::RangeUsed;

#[derive(Clone)]
pub(crate) struct RangeMerger {
    input: Arc<Mutex<Vec<PuzzleValueHolder<RangeUsed<f64>>>>>,
    output: PuzzlePiece<RangeUsed<f64>>
}

impl RangeMerger {
    pub(crate) fn new(puzzle: &PuzzleBuilder) -> RangeMerger {
        let mut output = puzzle.new_piece();
        #[cfg(debug_assertions)]
        output.set_name("RangeMerger");
        let output2 = output.clone();
        let input = Arc::new(Mutex::new(vec![]));
        let input2 = input.clone();
        puzzle.add_ready(move |_| {
            let dependencies = lock!(input2).iter().map(|x : &PuzzleValueHolder<_>| x.dependency()).collect::<Vec<_>>();
            output2.add_solver(&dependencies,move |solution| {
                Some(lock!(input2).iter().map(|x| x.get_clone(solution))
                    .fold(RangeUsed::None, |x,y| {
                        x.merge(&y)
                    }))
            });
        });
        RangeMerger {
            input,
            output
        }
    }

    pub(crate) fn add(&self, range: &PuzzleValueHolder<RangeUsed<f64>>) {
        lock!(self.input).push(range.clone());
    }

    pub(crate) fn output(&self) -> PuzzleValueHolder<RangeUsed<f64>> {
        PuzzleValueHolder::new(self.output.clone())
    }
}
