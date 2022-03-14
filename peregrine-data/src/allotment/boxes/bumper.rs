use std::sync::{Arc, Mutex};

use js_sys::Intl::Collator;
use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleDependency, PuzzleBuilder}, lock};

use crate::{allotment::{core::{arbitrator::Arbitrator, rangeused::RangeUsed, allotmentmetadata2::AllotmentMetadata2Builder}, style::style::Padding, boxes::{boxtraits::Stackable}, tree::collisionalgorithm::{CollisionAlgorithmHolder, CollisionToken}}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::{Ranged, Coordinated}};

#[derive(Clone)]
pub struct Bumper(Padder<UnpaddedBumper>);

impl Bumper {
    pub fn new(puzzle: &PuzzleBuilder, coord_system: &CoordinateSystem, padding: &Padding, metadata: &mut AllotmentMetadata2Builder) -> Bumper {
        Bumper(Padder::new(puzzle,coord_system,padding,metadata,|info| UnpaddedBumper::new(puzzle,info)))        
    }

    pub fn add_child<F>(&mut self, child: &F) where F: Stackable + Ranged {
        self.0.child_mut().add_child(child)
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
}

#[derive(Clone)]
pub struct UnpaddedBumper {
    puzzle: PuzzleBuilder,
    algorithm: CollisionAlgorithmHolder,
    info: PadderInfo,
    tokens: Arc<Mutex<Vec<PuzzleValueHolder<CollisionToken>>>>
}

impl UnpaddedBumper {
    pub fn new(puzzle: &PuzzleBuilder, info: &PadderInfo) -> UnpaddedBumper {
        let algorithm = CollisionAlgorithmHolder::new();
        let algorithm2 = algorithm.clone();
        info.child_height.add_solver(&[], move |_solution| {
            let height = algorithm2.bump();
            Some(height)
        });
        UnpaddedBumper {
            puzzle: puzzle.clone(),
            algorithm,
            info: info.clone(),
            tokens: Arc::new(Mutex::new(vec![]))
        }
    }

    fn make_token<F>(&mut self, child: &F) -> PuzzleValueHolder<CollisionToken> where F: Stackable + Ranged {
        let height = child.height();
        let full_range = child.full_range();     
        let piece = self.puzzle.new_piece(None);
        let algorithm = self.algorithm.clone();
        piece.add_solver(&[height.dependency(),full_range.dependency()], move |solution| {
            Some(algorithm.add_entry(&full_range.get_clone(solution),height.get_clone(solution)))
        });
        PuzzleValueHolder::new(piece)
    }

    fn set_child_top<F>(&mut self, child: &F, token: &PuzzleValueHolder<CollisionToken>) where F: Stackable {
        let child_top = self.puzzle.new_piece(None);
        let token = token.clone();
        let top = self.info.draw_top.clone();
        /* dependency on child_height ensures bumping is run so that token is not None */
        child_top.add_solver(&[token.dependency(),top.dependency(),self.info.child_height.dependency()], move |solution| {
            Some(token.get_clone(solution).get() + top.get_clone(solution))
        });
        child.set_top(&PuzzleValueHolder::new(child_top));
    }

    pub fn add_child<F>(&mut self, child: &F) where F: Stackable + Ranged {
        let token = self.make_token(child);
        self.set_child_top(child,&token);
        child.set_indent(&self.info.indent);
        lock!(self.tokens).push(token);
    }
}
