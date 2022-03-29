use std::sync::Arc;

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzlePiece, PuzzleValue, ClonablePuzzleValue, PuzzleBuilder, DerivedPuzzlePiece}};

use crate::{allotment::{core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::{boxtraits::Stackable}, util::{rangeused::RangeUsed, collisionalgorithm::{CollisionAlgorithmHolder}}}, CoordinateSystem};

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
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn priority(&self) -> i64 { self.0.priority() }
    fn set_top(&self, value: &PuzzleValueHolder<f64>) { self.0.set_top(value); }
    fn top_anchor(&self, puzzle: &PuzzleBuilder) -> PuzzleValueHolder<f64> { self.0.top_anchor(puzzle) }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
}

#[derive(Clone)]
struct BumpItem {
    name: AllotmentName,
    range: PuzzleValueHolder<RangeUsed<f64>>,
    height: PuzzleValueHolder<f64>,
    top: PuzzlePiece<f64>
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

    fn build_reduce(&mut self, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        let mut dependencies = vec![self.algorithm.dependency()];
        let mut items = vec![];
        for (child,size) in children {
            let mut child_top = self.puzzle.new_piece();
            #[cfg(debug_assertions)]
            child_top.set_name("bumper/child_top");
            let child_top2 = child_top.clone();
            items.push(BumpItem {
                name: size.name.clone(),
                range: size.range.clone(),
                height: size.height.clone(),
                top: child_top
            });
            child.set_top(&PuzzleValueHolder::new(child_top2.clone()));
            dependencies.push(size.range.dependency());
            dependencies.push(size.height.dependency());
        }
        let all_items = Arc::new(items);
        let all_items2 = all_items.clone();
        let mut solved = self.puzzle.new_piece();
        let algorithm = self.algorithm.clone();
        #[cfg(debug_assertions)]
        solved.set_name("bumper/solved");
        solved.add_solver(&dependencies, move |solution| {
            let algorithm = algorithm.get(solution);
            for item in all_items2.iter() {
                algorithm.add_entry(&item.name,&item.range.get_clone(solution),item.height.get_clone(solution));
            }
            algorithm.bump();
            Some(algorithm)
        });
        for item in all_items.iter() {
            let item2 = item.clone();
            let our_top2 = self.our_top.clone();
            let algorithm = self.algorithm.clone();
            item.top.add_solver(&[self.our_top.dependency(),solved.dependency()], move |solution| {
                Some(our_top2.get_clone(solution) + algorithm.get(solution).get(&item2.name))
            });
        }
        PuzzleValueHolder::new(DerivedPuzzlePiece::new(solved, |solved| solved.height()))
    }
}
