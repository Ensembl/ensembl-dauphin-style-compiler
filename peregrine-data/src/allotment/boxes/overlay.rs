use std::sync::{Arc, Mutex};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder}, lock};

use crate::{allotment::{core::{arbitrator::Arbitrator, allotmentmetadata2::AllotmentMetadata2Builder}, style::style::Padding, boxes::boxtraits::Stackable}, AllotmentMetadata, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::Coordinated};

#[derive(Clone)]
pub struct Overlay(Padder<UnpaddedOverlay>);

impl Overlay {
    pub fn new(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Overlay {
        Overlay(Padder::new(puzzle,coord_system,padding,metadata,|info| UnpaddedOverlay::new(puzzle,info)))
    }

    pub fn add_child(&mut self, child: &dyn Stackable) {
        self.0.child_mut().add_child(child)
    }
}

#[derive(Clone)]
struct UnpaddedOverlay {
    info: PadderInfo,
    kid_heights: Arc<Mutex<Vec<PuzzleValueHolder<f64>>>>
}

impl UnpaddedOverlay {
    fn new(puzzle: &PuzzleBuilder, info: &PadderInfo) -> UnpaddedOverlay {
        let kid_heights = Arc::new(Mutex::new(vec![]));
        let kid_heights2 = kid_heights.clone();
        let height2 = info.child_height.clone();
        info.child_height.add_ready(move |_| {
            let deps = lock!(kid_heights2).iter().map(|x : &PuzzleValueHolder<f64>| x.dependency()).collect::<Vec<_>>();
            height2.add_solver(&deps, move |solution| {
                let height = lock!(kid_heights2).iter()
                    .map(|p| p.get_clone(solution))
                    .fold(0., f64::max);
                Some(height)
            })
        });
        UnpaddedOverlay { info: info.clone(), kid_heights }
    }

    fn add_child(&mut self, child: &dyn Stackable) {
        child.set_top(&self.info.draw_top);
        child.set_indent(&self.info.indent);
        lock!(self.kid_heights).push(child.height());
    }
}

impl Coordinated for Overlay {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Overlay {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn set_indent(&self, value: &PuzzleValueHolder<f64>) { self.0.set_indent(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
}
