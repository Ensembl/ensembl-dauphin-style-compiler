use std::{sync::{Arc, Mutex}, borrow::Borrow};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleDependency, PuzzleBuilder}, lock, log};

use crate::{allotment::{core::{arbitrator::Arbitrator, rangeused::RangeUsed, allotmentmetadata2::AllotmentMetadata2Builder}, style::{style::Padding}, boxes::{boxtraits::Stackable}, tree::collisionalgorithm::{CollisionAlgorithmHolder, CollisionToken}}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::{Coordinated, StackableAddable}, rangecontainer::{RangeMerger}};

#[derive(Clone)]
pub struct Bumper(Padder<UnpaddedBumper>);

impl Bumper {
    pub fn new(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Bumper {
        Bumper(Padder::new(puzzle,coord_system,padding,metadata,|info| UnpaddedBumper::new(puzzle,info,false)))        
    }

    pub fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child,0)
    }
}

impl Coordinated for Bumper {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Bumper {
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn height(&self) -> PuzzleValueHolder<f64> { self.0.height() }
    fn set_indent(&self, value: &PuzzleValueHolder<f64>) { self.0.set_indent(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { self.0.full_range() }
}

#[derive(Clone)]
pub struct UnpaddedBumper {
    puzzle: PuzzleBuilder,
    algorithm: CollisionAlgorithmHolder,
    info: PadderInfo,
    tokens: Arc<Mutex<Vec<PuzzleValueHolder<CollisionToken>>>>,
    ranges: Arc<Mutex<Option<RangeMerger>>>
}

impl UnpaddedBumper {
    pub fn new(puzzle: &PuzzleBuilder, info: &PadderInfo, keep_range: bool) -> UnpaddedBumper {
        let algorithm = CollisionAlgorithmHolder::new();
        let algorithm2 = algorithm.clone();
        let child_height = info.child_height.clone();
        let tokens =  Arc::new(Mutex::new(vec![]));
        let tokens2 = tokens.clone();
        puzzle.add_ready(move |_| {
            let dependencies = lock!(tokens2).iter().map(|x : &PuzzleValueHolder<CollisionToken>| x.dependency()).collect::<Vec<_>>();
            child_height.add_solver(&dependencies, move |_solution| {
                let height = algorithm2.bump();
                Some(height)
            });
        });
        UnpaddedBumper {
            puzzle: puzzle.clone(),
            algorithm,
            info: info.clone(),
            tokens,
            ranges: Arc::new(Mutex::new(if keep_range { Some(RangeMerger::new(puzzle)) } else { None }))
        }
    }

    fn make_token(&mut self, child: &dyn Stackable) -> PuzzleValueHolder<CollisionToken> {
        let height = child.height();
        let full_range = child.full_range();     
        let mut piece = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        piece.set_name("bumper/make_token");
        let algorithm = self.algorithm.clone();
        piece.add_solver(&[height.dependency(),full_range.dependency()], move |solution| {
            Some(algorithm.add_entry(&full_range.get_clone(solution),height.get_clone(solution)))
        });
        PuzzleValueHolder::new(piece)
    }

    fn set_child_top(&mut self, child: &dyn Stackable, token: &PuzzleValueHolder<CollisionToken>) {
        let mut child_top = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        child_top.set_name("bumper/child_top");
        let token = token.clone();
        let top = self.info.draw_top.clone();
        /* dependency on child_height ensures bumping is run so that token is not None */
        child_top.add_solver(&[token.dependency(),top.dependency(),self.info.child_height.dependency()], move |solution| {
            Some(token.get_clone(solution).get() + top.get_clone(solution))
        });
        child.set_top(&PuzzleValueHolder::new(child_top));
    }
}

impl StackableAddable for UnpaddedBumper {
    fn add_child(&mut self, child: &dyn Stackable, _priority: i64) {
        let token = self.make_token(child);
        self.set_child_top(child,&token);
        child.set_indent(&self.info.indent);
        lock!(self.tokens).push(token);
        if let Some(ranges) = &*lock!(self.ranges) {
            ranges.add(&child.full_range());
        }
    }
}
