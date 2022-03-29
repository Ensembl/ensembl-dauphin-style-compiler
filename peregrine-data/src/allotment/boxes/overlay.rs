use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, CommutingSequence}};

use crate::{allotment::{core::{ aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart}}, boxes::boxtraits::Stackable, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo, PadderSpecifics}, boxtraits::{Coordinated, BuildSize }};

#[derive(Clone)]
pub struct Overlay(Padder);

impl Overlay {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Overlay {
        Overlay(Padder::new(prep,name,style,aligner,|prep,info| Box::new(UnpaddedOverlay::new(&prep.puzzle,info))))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
    }
}

#[derive(Clone)]
struct UnpaddedOverlay {
    puzzle: PuzzleBuilder,
    info: PadderInfo
}

impl UnpaddedOverlay {
    fn new(puzzle: &PuzzleBuilder, info: &PadderInfo,) -> UnpaddedOverlay {
        UnpaddedOverlay { 
            puzzle: puzzle.clone(),
            info: info.clone()
        }
    }
}

impl Coordinated for Overlay {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Overlay {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn priority(&self) -> i64 { self.0.priority() }
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
}

impl PadderSpecifics for UnpaddedOverlay {
    fn cloned(&self) -> Box<dyn PadderSpecifics> { Box::new(self.clone()) }

    fn add_child(&mut self, child: &dyn Stackable) {
        //StackableAddable::add_child(self,child,priority);
    }

    fn build_reduce(&mut self, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        let mut max_height = CommutingSequence::new(0.,|a,b| { f64::max(*a,*b) });
        for (child,size) in children {
            child.set_top(&self.info.draw_top);
            max_height.add(&size.height);
        }
        max_height.build(&self.puzzle)
    }
}
