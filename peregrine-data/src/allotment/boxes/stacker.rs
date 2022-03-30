use std::sync::Arc;

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleBuilder, CommutingSequence, DelayedPuzzleValue, compose2, build_puzzle_vec, DerivedPuzzlePiece}};

use crate::{allotment::{style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::boxtraits::Stackable, core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}}, CoordinateSystem};

use super::{container::{Container}, boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

#[derive(Clone)]
pub struct Stacker(Container);

impl Stacker {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Stacker {
        Stacker(Container::new(prep,name,style,aligner,UnpaddedStacker::new(&prep.puzzle)))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
    }
}

#[derive(Clone)]
struct AddedChild {
    priority: i64,
    height: PuzzleValueHolder<f64>
}

fn child_tops(puzzle: &PuzzleBuilder, children: &[AddedChild]) -> (PuzzleValueHolder<Vec<f64>>,PuzzleValueHolder<f64>) {
    let mut children = children.iter().enumerate().collect::<Vec<_>>();
    let mut self_height = CommutingSequence::new(0.,|a,b| *a+*b);
    children.sort_by_cached_key(|c| c.1.priority);
    let positions = Arc::new(children.iter().map(|c| c.0).collect::<Vec<_>>());
    let heights = children.iter().map(|c| c.1.height.clone()).collect::<Vec<_>>();
    for child_height in &heights {
        self_height.add(&child_height);
    }
    let heights = build_puzzle_vec(puzzle,&heights);
    /* calculate our own height */
    let self_height = self_height.build(puzzle);
    /* set relative tops */
    let relative_tops = DerivedPuzzlePiece::new(heights,move |heights| {
        let mut tops = vec![];
        let mut top = 0.;
        for height in heights {
            tops.push(top);
            top += *height.as_ref();
        }
        let mut out = vec![0.;tops.len()];
        for (i,pos) in positions.iter().enumerate() {
            out[*pos] = tops[i];
        }
        out
    });
    (PuzzleValueHolder::new(relative_tops),PuzzleValueHolder::new(self_height))
}

#[derive(Clone)]
struct UnpaddedStacker {
    relative_tops: DelayedPuzzleValue<Vec<f64>>
}

impl UnpaddedStacker {
    fn new(puzzle: &PuzzleBuilder) -> UnpaddedStacker {
        UnpaddedStacker {
            relative_tops: DelayedPuzzleValue::new(puzzle)
        }
    }
}

impl Stackable for Stacker {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>) { self.0.locate(prep,top); }
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
}

impl Coordinated for Stacker {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl ContainerSpecifics for UnpaddedStacker {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, prep: &mut CarriageUniversePrep, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        let mut added = vec![];
        for (child,size) in children {
            added.push(AddedChild {
                height: size.height.clone(),
                priority: child.priority()
            });
        }
        let (relative_tops,self_height) = child_tops(&prep.puzzle,&added);
        self.relative_tops.set(&prep.puzzle, relative_tops);
        self_height
    }

    fn set_locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for (i,child) in children.iter_mut().enumerate() {
            let relative_top = DerivedPuzzlePiece::new(self.relative_tops.clone(),move |tops| tops[i]);
            let abs_top = compose2(&prep.puzzle,top,&PuzzleValueHolder::new(relative_top),|a,b| *a+*b);
            child.locate(prep,&abs_top);
        }
    }
}
