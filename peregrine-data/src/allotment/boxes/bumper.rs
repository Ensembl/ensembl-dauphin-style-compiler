use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleBuilder}};

use crate::{allotment::{core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart}}, boxes::{boxtraits::Stackable}, util::{rangeused::RangeUsed, collisionalgorithm::{CollisionToken, CollisionAlgorithmHolder}}}, CoordinateSystem};

use super::{padder::{Padder, PadderInfo, PadderSpecifics}, boxtraits::{Coordinated, BuildSize}};

#[derive(Clone)]
pub struct Bumper(Padder);

impl Bumper {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Bumper {
        Bumper(Padder::new(prep,name,style,aligner,|prep,info| Box::new(UnpaddedBumper::new(&prep.puzzle,info))))
    }

    pub(crate) fn add_child(&mut self, child: &dyn Stackable) {
        self.0.add_child(child)
    }
}

impl Coordinated for Bumper {
    fn coordinate_system(&self) -> &CoordinateSystem { self.0.coordinate_system() }
}

impl Stackable for Bumper {
    fn cloned(&self) -> Box<dyn Stackable> { Box::new(self.clone()) }
    fn priority(&self) -> i64 { self.0.priority() }
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
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
    algorithm: PuzzlePiece<CollisionAlgorithmHolder>,
    our_top: PuzzleValueHolder<f64>
}

impl UnpaddedBumper {
    pub fn new(puzzle: &PuzzleBuilder, info: &PadderInfo) -> UnpaddedBumper {
        let mut algorithm =  puzzle.new_piece();
        #[cfg(debug_assertions)]
        algorithm.set_name("algorithm");
        algorithm.add_solver(&[], |_| {
            Some(CollisionAlgorithmHolder::new())
        });
        UnpaddedBumper {
            puzzle: puzzle.clone(),
            algorithm: algorithm.clone(),
            our_top: info.draw_top.clone(),
        }
    }
}

impl PadderSpecifics for UnpaddedBumper {
    fn cloned(&self) -> Box<dyn PadderSpecifics> { Box::new(self.clone()) }

    fn add_child(&mut self, child: &dyn Stackable) {
        //StackableAddable::add_child(self,child,priority);
    }

    fn build_reduce(&mut self, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        let mut items = vec![];
        for (child,size) in children {
            let mut child_top = self.puzzle.new_piece();
            #[cfg(debug_assertions)]
            child_top.set_name("bumper/child_top");
            let mut token = self.puzzle.new_piece();
            #[cfg(debug_assertions)]
            token.set_name("bumper/token");
            let child_top2 = child_top.clone();
            items.push(BumpItem {
                range: size.range.clone(),
                height: size.height.clone(),
                top: child_top,
                token
            });
            child.set_top(&PuzzleValueHolder::new(child_top2.clone()));
        }
        let mut dependencies = vec![];
        for item in &*items {
            let algorithm = self.algorithm.clone();
            let algorithm2 = algorithm.clone();
            let item2 = item.clone();
            item.token.add_solver(&[algorithm.dependency(),item.range.dependency(),item.height.dependency()], move |solution| {
                let algorithm = &algorithm2.get(solution);
                Some(algorithm.add_entry(&item2.range.get_clone(solution),item2.height.get_clone(solution)))
            });
            dependencies.push(item.token.dependency());
        }
        dependencies.push(self.algorithm.dependency());
        let algorithm2 = self.algorithm.clone();
        let mut solved = self.puzzle.new_piece();
        #[cfg(debug_assertions)]
        solved.set_name("bumper/solved");
        solved.add_solver(&dependencies, move |solution| {
            Some(algorithm2.get(solution).bump())
        });
        for item in &*items {
            let item2 = item.clone();
            let our_top2 = self.our_top.clone();
            item.top.add_solver(&[self.our_top.dependency(),item.token.dependency(),solved.dependency()], move |solution| {
                Some(our_top2.get_clone(solution) + item2.token.get(solution).get())
            });
        }
        PuzzleValueHolder::new(solved)
    }
}
