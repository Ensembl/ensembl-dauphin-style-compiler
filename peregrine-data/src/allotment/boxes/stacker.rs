use std::{sync::{Arc, Mutex}, mem};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, ClonablePuzzleValue, PuzzleValue, PuzzleBuilder, ConstantPuzzlePiece}, lock, log};

use crate::{allotment::{style::style::Padding, boxes::boxtraits::Stackable, core::allotmentmetadata2::AllotmentMetadata2Builder}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::Coordinated};

#[derive(Clone)]
pub struct Stacker(Padder<UnpaddedStacker>);

impl Stacker {
    pub fn new(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Stacker {
        Stacker(Padder::new(puzzle,coord_system,padding,metadata,|info| UnpaddedStacker::new(puzzle,info)))
    }

    pub fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        self.0.child_mut().add_child(child,priority)
    }
}

struct AddedChild {
    top: PuzzlePiece<f64>,
    bottom: PuzzlePiece<f64>,
    priority: i64
}

struct AddedChildren {
    children: Vec<AddedChild>
}

impl AddedChildren {
    fn new() -> AddedChildren {
        AddedChildren {
            children: vec![]
        }
    }

    fn add_child(&mut self, puzzle: &mut PuzzleBuilder, top: &PuzzlePiece<f64>, height: &PuzzleValueHolder<f64>, priority: i64) {
        let mut bottom = puzzle.new_piece();
        #[cfg(debug_assertions)]
        bottom.set_name("bottom");
        let top2 = top.clone();
        let height2 = height.clone();
        bottom.add_solver(&[top.dependency(),height.dependency()], move |solution| {
            Some(top2.get_clone(solution) + height2.get_clone(solution))
        });
        self.children.push(AddedChild {
            top: top.clone(),
            bottom, priority
        })
    }

    fn compute(&mut self, top: PuzzleValueHolder<f64>) -> PuzzleValueHolder<f64> {
        let mut bottom = top;
        self.children.sort_by_cached_key(|c| c.priority);
        for child in &self.children {
            let bottom2 = bottom.clone();
            child.top.add_solver(&[bottom.dependency()], move |solution| {
                Some(bottom2.get_clone(solution))
            });
            bottom = PuzzleValueHolder::new(child.bottom.clone());
        }
        bottom
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
        let total_height = padder_info.child_height.clone();
        let total_height2 = total_height.clone();
        let children = Arc::new(Mutex::new(AddedChildren::new()));
        let children2 = children.clone();
        total_height.add_ready(move |_| {
            let top2 = top.clone();
            let height = lock!(children2).compute(top.clone());
            total_height2.add_solver(&[height.dependency(),top.dependency()], move |solution| {
                Some(height.get_clone(solution) - top2.get_clone(solution))
            })
        });
        UnpaddedStacker { puzzle: puzzle.clone(), padder_info: padder_info.clone(), children }
    }

    fn add_child(&mut self, child: &dyn Stackable, priority: i64) {
        let mut top = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        top.set_name("stacker/add_child");
        child.set_top(&PuzzleValueHolder::new(top.clone()));
        lock!(self.children).add_child(&mut self.puzzle,&top,&child.height(),priority);
        child.set_indent(&PuzzleValueHolder::new(self.padder_info.indent.clone()));
    }
}

impl Stackable for Stacker {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn set_indent(&self, value: &PuzzleValueHolder<f64>) { self.0.set_indent(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
}

impl Coordinated for Stacker {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}
