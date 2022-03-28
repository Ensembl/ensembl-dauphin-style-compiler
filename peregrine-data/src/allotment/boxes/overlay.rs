use std::sync::{Arc, Mutex};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder}, lock};

use crate::{allotment::{core::{ aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::{Coordinated, StackableAddable }};

#[derive(Clone)]
pub struct Overlay(Padder<UnpaddedOverlay>);

impl Overlay {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Overlay {
        Overlay(Padder::new(prep,name,style,aligner,|prep,info| UnpaddedOverlay::new(&prep.puzzle,info)))
    }

    pub fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child,0)
    }
}

#[derive(Clone)]
struct UnpaddedOverlay {
    info: PadderInfo,
    kid_heights: Arc<Mutex<Vec<PuzzleValueHolder<f64>>>>,
}

impl UnpaddedOverlay {
    fn new(_puzzle: &PuzzleBuilder, info: &PadderInfo,) -> UnpaddedOverlay {
        let kid_heights = Arc::new(Mutex::new(vec![]));
        let kid_heights2 = kid_heights.clone();
        let mut height2 = info.child_height.clone();
        #[cfg(debug_assertions)]
        height2.set_name("ch in overlay");
        info.child_height.add_ready(move |_,_| {
            let deps = lock!(kid_heights2).iter().map(|x : &PuzzleValueHolder<f64>| x.dependency()).collect::<Vec<_>>();
            height2.add_solver(&deps, move |solution| {
                let height = lock!(kid_heights2).iter()
                    .map(|p| p.get_clone(solution))
                    .fold(0., f64::max);
                Some(height)
            })
        });
        UnpaddedOverlay { 
            info: info.clone(), 
            kid_heights
        }
    }
}

impl StackableAddable for UnpaddedOverlay {
    fn add_child(&mut self, child: &dyn Stackable, _priority: i64) {
        child.set_top(&self.info.draw_top);
        lock!(self.kid_heights).push(child.height());
    }
}

impl Coordinated for Overlay {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Overlay {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { self.0.full_range() }
}
