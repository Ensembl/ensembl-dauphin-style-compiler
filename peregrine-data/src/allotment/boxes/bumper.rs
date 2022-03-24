use std::{sync::{Arc, Mutex}};

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleBuilder}, lock };

use crate::{allotment::{core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart}}, boxes::{boxtraits::Stackable}, util::{rangeused::RangeUsed, collisionalgorithm::{CollisionToken, CollisionAlgorithmHolder}}}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo}, boxtraits::{Coordinated, StackableAddable}};

#[derive(Clone)]
pub struct Bumper(Padder<UnpaddedBumper>);

impl Bumper {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Bumper {
        Bumper(Padder::new(prep,name,style,aligner,|prep,info| UnpaddedBumper::new(&prep.puzzle,info)))
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
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn full_range(&self) -> PuzzleValueHolder<RangeUsed<f64>> { self.0.full_range() }
}

#[derive(Clone)]
struct BumpItem {
    range: PuzzleValueHolder<RangeUsed<f64>>,
    height: PuzzleValueHolder<f64>,
    top: PuzzlePiece<f64>,
    token: PuzzlePiece<CollisionToken>
}

#[derive(Clone)]
pub struct UnpaddedBumper {
    puzzle: PuzzleBuilder,
    items: Arc<Mutex<Vec<BumpItem>>>,
}

impl UnpaddedBumper {
    pub fn new(puzzle: &PuzzleBuilder, info: &PadderInfo) -> UnpaddedBumper {
        let our_top = info.draw_top.clone();
        let mut algorithm =  puzzle.new_piece();
        #[cfg(debug_assertions)]
        algorithm.set_name("algorithm");
        algorithm.add_solver(&[], |_| {
            Some(CollisionAlgorithmHolder::new())
        });
        let items  = Arc::new(Mutex::new(Vec::<BumpItem>::new()));
        let solved = info.child_height.clone();
        let items2 = items.clone();
        puzzle.add_ready(move |_| {
            let items = lock!(items2);
            let mut dependencies = vec![];
            for item in &*items {
                let algorithm2 = algorithm.clone();
                let item2 = item.clone();
                item.token.add_solver(&[algorithm.dependency(),item.range.dependency(),item.height.dependency()], move |solution| {
                    let algorithm = &algorithm2.get(solution);
                    Some(algorithm.add_entry(&item2.range.get_clone(solution),item2.height.get_clone(solution)))
                });
                dependencies.push(item.token.dependency());
            }
            dependencies.push(algorithm.dependency());
            let algorithm2 = algorithm.clone();
            solved.add_solver(&dependencies, move |solution| {
                Some(algorithm2.get(solution).bump())
            });
            for item in &*items {
                let item2 = item.clone();
                let our_top2 = our_top.clone();
                item.top.add_solver(&[our_top.dependency(),item.token.dependency(),solved.dependency()], move |solution| {
                    Some(our_top2.get_clone(solution) + item2.token.get(solution).get())
                });
            }
        });
        UnpaddedBumper {
            puzzle: puzzle.clone(),
            items
        }
    }

    fn set_child_top(&mut self, child: &dyn Stackable) {
        let mut child_top = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        child_top.set_name("bumper/child_top");
        let mut token = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        token.set_name("bumper/token");
        let child_top2 = child_top.clone();
        lock!(self.items).push(BumpItem {
            range: child.full_range(),
            height: child.height(),
            top: child_top,
            token
        });
        child.set_top(&PuzzleValueHolder::new(child_top2.clone()));
    }
}

impl StackableAddable for UnpaddedBumper {
    fn add_child(&mut self, child: &dyn Stackable, _priority: i64) {
        self.set_child_top(child);
    }
}
