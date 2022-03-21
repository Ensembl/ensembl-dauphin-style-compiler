use std::{sync::{Arc, Mutex}, mem};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder, ConstantPuzzlePiece, FoldValue}, lock, log};

use crate::{allotment::{style::{style::Padding}, boxes::boxtraits::Stackable, core::{allotmentmetadata2::AllotmentMetadata2Builder, rangeused::RangeUsed}}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::{Coordinated, StackableAddable}};

#[derive(Clone)]
pub struct Stacker(Padder<UnpaddedStacker>);

impl Stacker {
    pub fn new(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Stacker {
        Stacker(Padder::new(puzzle,coord_system,padding,metadata,|info| UnpaddedStacker::new(puzzle,info)))
    }

    pub fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        self.0.add_child(child,priority)
    }
}

struct AddedChild {
    priority: i64,
    top: PuzzlePiece<f64>,
    height: PuzzleValueHolder<f64>
}

struct AddedChildren {
    children: Vec<AddedChild>,
    self_top: PuzzleValueHolder<f64>,
    self_height: FoldValue<f64>,
    relative_tops: PuzzlePiece<Vec<f64>>
}

impl AddedChildren {
    fn new(puzzle: &PuzzleBuilder, top: &PuzzleValueHolder<f64>, height: &PuzzlePiece<f64>) -> AddedChildren {
        let mut relative_tops = puzzle.new_piece();
        #[cfg(debug_assertions)]
        relative_tops.set_name("relative_tops");
        AddedChildren {
            children: vec![],
            self_top: top.clone(),
            self_height: FoldValue::new(height.clone(),|a,b| a+b),
            relative_tops
        }
    }

    fn add_child(&mut self, puzzle: &mut PuzzleBuilder, top: &PuzzlePiece<f64>, height: &PuzzleValueHolder<f64>, priority: i64) {
        self.children.push(AddedChild {
            priority,
            top: top.clone(),
            height: height.clone()
        });
    }

    fn ready(&mut self) {
        self.children.sort_by_cached_key(|c| c.priority);
        let heights = self.children.iter().map(|c| c.height.clone()).collect::<Vec<_>>();
        /* calculate our own height */
        for child_height in &heights {
            self.self_height.add(&child_height);
        }
        self.self_height.build();
        /* set relative tops */
        let height_deps = heights.iter().map(|x| x.dependency().clone()).collect::<Vec<_>>();
        self.relative_tops.add_solver(&height_deps, move |solution| {
            let mut tops = vec![];
            let mut top = 0.;
            for height in &heights {
                tops.push(top);
                top += height.get_clone(solution);
            }
            Some(tops)
        });
        /* set child tops */
        for (i,child) in self.children.iter().enumerate() {
            let self_top = self.self_top.clone();
            let children_before = i;
            let relative_tops = self.relative_tops.clone();
            child.top.add_solver(&[self.self_top.dependency(),self.relative_tops.dependency()], move |solution| {
                Some(self_top.get_clone(solution) + relative_tops.get_clone(solution)[children_before])
            });
        }
    }
}

#[derive(Clone)]
struct UnpaddedStacker {
    puzzle: PuzzleBuilder,
    padder_info: PadderInfo,
    children: Arc<Mutex<AddedChildren>>
}

impl UnpaddedStacker {
    fn new(puzzle: &PuzzleBuilder, padder_info: &PadderInfo) -> UnpaddedStacker {
        let top = padder_info.draw_top.clone();
        let children = Arc::new(Mutex::new(AddedChildren::new(puzzle,&top,&padder_info.child_height)));
        let children2 = children.clone();
        puzzle.add_ready(move |_| {
            lock!(children2).ready();
        });
        UnpaddedStacker { puzzle: puzzle.clone(), padder_info: padder_info.clone(), children }
    }
}

impl StackableAddable for UnpaddedStacker {
    fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        let mut top = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        top.set_name("stacker/add_child");
        child.set_top(&PuzzleValueHolder::new(top.clone()));
        lock!(self.children).add_child(&mut self.puzzle,&top,&child.height(),priority);
    }
}

impl Stackable for Stacker {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { self.0.full_range() }
}

impl Coordinated for Stacker {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}
