use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder, FoldValue, ConstantPuzzlePiece, CommutingSequence, DelayedPuzzleValue, compose2, build_puzzle_vec, DerivedPuzzlePiece}, lock, log};

use crate::{allotment::{style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart}}, boxes::boxtraits::Stackable, core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, util::rangeused::RangeUsed}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo, PadderSpecifics}, boxtraits::{Coordinated, BuildSize}};

#[derive(Clone)]
pub struct Stacker(Padder);

impl Stacker {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Stacker {
        Stacker(Padder::new(prep,name,style,aligner,|prep,info| Box::new(UnpaddedStacker::new(&prep.puzzle,info))))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
    }
}

struct AddedChild {
    priority: i64,
    top: DelayedPuzzleValue<f64>,
    height: PuzzleValueHolder<f64>
}

struct AddedChildren {
    children: Vec<AddedChild>,
    self_top: PuzzleValueHolder<f64>
}

impl AddedChildren {
    fn new(puzzle: &PuzzleBuilder, top: &PuzzleValueHolder<f64>) -> AddedChildren {
        AddedChildren {
            children: vec![],
            self_top: top.clone()
        }
    }

    fn add_child(&mut self, _puzzle: &mut PuzzleBuilder, top: &DelayedPuzzleValue<f64>, height: &PuzzleValueHolder<f64>, priority: i64) {
        self.children.push(AddedChild {
            priority,
            top: top.clone(),
            height: height.clone()
        });
    }

    fn ready(&mut self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> {
        let mut self_height = CommutingSequence::new(0.,|a,b| *a+*b);
        self.children.sort_by_cached_key(|c| c.priority);
        let heights = self.children.iter().map(|c| c.height.clone()).collect::<Vec<_>>();
        for child_height in &heights {
            self_height.add(&child_height);
        }
        let heights = build_puzzle_vec(puzzle,&heights);
        /* calculate our own height */
        let self_height = self_height.build(puzzle);
        /* set relative tops */

        let relative_tops = DerivedPuzzlePiece::new(heights,|heights| {
            let mut tops = vec![];
            let mut top = 0.;
            for height in heights {
                tops.push(top);
                top += *height.as_ref();
            }
            tops
        });
        /* set child tops */
        for (i,child) in self.children.iter().enumerate() {
            let children_before = i;
            child.top.set(puzzle,compose2(puzzle,&self.self_top,&PuzzleValueHolder::new(relative_tops.clone()),move |a,b| {
                a+b[children_before]
            }));
        }
        self_height
    }
}

#[derive(Clone)]
struct UnpaddedStacker {
    puzzle: PuzzleBuilder,
    top: PuzzleValueHolder<f64>
}

impl UnpaddedStacker {
    fn new(puzzle: &PuzzleBuilder, padder_info: &PadderInfo) -> UnpaddedStacker {
        UnpaddedStacker {
            puzzle: puzzle.clone(),
            top: padder_info.draw_top.clone()
        }
    }
}

impl Stackable for Stacker {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn priority(&self) -> i64 { self.0.priority() }
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
}

impl Coordinated for Stacker {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl PadderSpecifics for UnpaddedStacker {
    fn cloned(&self) -> Box<dyn PadderSpecifics> { Box::new(self.clone()) }

    fn add_child(&mut self, child: &dyn Stackable) {
        //StackableAddable::add_child(self,child,priority);
    }

    fn build_reduce(&mut self, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        let mut added = AddedChildren::new(&self.puzzle,&self.top);
        for (child,size) in children {
            let top = DelayedPuzzleValue::new(&self.puzzle);
            added.add_child(&mut self.puzzle,&top,&size.height,child.priority());    
            child.set_top(&PuzzleValueHolder::new(top.clone()));
        }
        added.ready(&self.puzzle)
    }
}
