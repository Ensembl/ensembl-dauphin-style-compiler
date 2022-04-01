use std::sync::Arc;

use peregrine_toolkit::{puzzle::{PuzzleValueHolder, PuzzleValue, ClonablePuzzleValue, PuzzleBuilder, DerivedPuzzlePiece, DelayedPuzzleValue, compose2}};

use crate::{allotment::{core::{aligner::Aligner, carriageuniverse::CarriageUniversePrep}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::{boxtraits::Stackable}, util::{rangeused::RangeUsed}, collision::collisionalgorithm::CollisionAlgorithm}, CoordinateSystem};

use super::{container::{Container}, boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

#[derive(Clone)]
pub struct Bumper(Container);

impl Bumper {
    pub(crate) fn new(prep: &mut CarriageUniversePrep, name: &AllotmentNamePart, style: &ContainerAllotmentStyle, aligner: &Aligner) -> Bumper {
        Bumper(Container::new(prep,name,style,aligner,UnpaddedBumper::new(&prep.puzzle,&AllotmentName::from_part(name))))
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
    fn locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>) { self.0.locate(prep,top); }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&mut self, prep: &mut CarriageUniversePrep) -> BuildSize { self.0.build(prep) }
}

#[derive(Clone)]
struct BumpItem {
    name: AllotmentName,
    range: PuzzleValueHolder<RangeUsed<f64>>,
    height: PuzzleValueHolder<f64>
}

#[derive(Clone)]
pub struct UnpaddedBumper {
    algorithm: DelayedPuzzleValue<CollisionAlgorithm>,
    name: AllotmentName
}

impl UnpaddedBumper {
    pub fn new(puzzle: &PuzzleBuilder, name: &AllotmentName) -> UnpaddedBumper {
        UnpaddedBumper {
            algorithm: DelayedPuzzleValue::new(&puzzle),
            name: name.clone()
        }
    }
}

impl ContainerSpecifics for UnpaddedBumper {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, prep: &mut CarriageUniversePrep, children: &[(&Box<dyn Stackable>,BuildSize)]) -> PuzzleValueHolder<f64> {
        /* build all_items, a solution-invariant structure of everything we need to bump each time */
        let mut dependencies = vec![];
        let mut items = vec![];
        for (_,size) in children {
            items.push(BumpItem {
                name: size.name.clone(),
                range: size.range.clone(),
                height: size.height.clone()
            });
            dependencies.push(size.range.dependency());
            dependencies.push(size.height.dependency());
        }
        let all_items = Arc::new(items);
        /* Get the bumper */
        let algorithm = prep.bumper_factory.get(&prep.puzzle,&self.name).clone();
        dependencies.push(algorithm.dependency());
        /* Create a piece which can bump everything in all_items each time and yield a CollisionAlgorithm */
        let all_items2 = all_items.clone();
        let mut solved = prep.puzzle.new_piece();
        #[cfg(debug_assertions)]
        solved.set_name("bumper/solved");
        solved.add_solver(&dependencies, move |solution| {
            let algorithm = algorithm.get_clone(solution);
            for item in all_items2.iter() {
                algorithm.add_entry(&item.name,&item.range.get_clone(solution),item.height.get_clone(solution));
            }
            algorithm.bump();
            Some(algorithm)
        });
        self.algorithm.set(&prep.puzzle,PuzzleValueHolder::new(solved.clone()));
        /* Cause algorithm to report how high we are per solution */
        PuzzleValueHolder::new(DerivedPuzzlePiece::new(solved, |solved| solved.height()))
    }

    fn set_locate(&mut self, prep: &mut CarriageUniversePrep, top: &PuzzleValueHolder<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for child in children.iter_mut() {
            /* Retrieve algorithm offset from bumper top */
            let name = child.name().clone();
            let offset = DerivedPuzzlePiece::new(self.algorithm.clone(),move |algorithm|
                algorithm.get(&name)
            );
            /* Add that to our reported top */
            child.locate(prep,&compose2(&prep.puzzle,top,&PuzzleValueHolder::new(offset),|a,b| *a+*b));
        }
    }
}
