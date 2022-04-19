use std::sync::Arc;

use peregrine_toolkit::{puzzle::{variable, derived, delayed, DelayedSetter, compose, short_memoized_arc, StaticValue, derived_debug}};

use crate::{allotment::{core::{carriageoutput::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}, allotmentname::{AllotmentNamePart, AllotmentName}}, boxes::{boxtraits::Stackable}, util::{rangeused::RangeUsed}, collision::{collisionalgorithm::CollisionAlgorithm}}, CoordinateSystem};

use super::{container::{Container}, boxtraits::{Coordinated, BuildSize, ContainerSpecifics}};

#[derive(Clone)]
pub struct Bumper(Container);

impl Bumper {
    pub(crate) fn new(name: &AllotmentNamePart, style: &ContainerAllotmentStyle) -> Bumper {
        Bumper(Container::new(name,style,UnpaddedBumper::new(&AllotmentName::from_part(name))))
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
    fn locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>) { self.0.locate(prep,top); }
    fn name(&self) -> &AllotmentName { self.0.name( )}
    fn priority(&self) -> i64 { self.0.priority() }
    fn build(&mut self, prep: &mut BoxPositionContext) -> BuildSize { self.0.build(prep) }
}

#[derive(Clone)]
struct BumpItem {
    name: AllotmentName,
    range: StaticValue<RangeUsed<f64>>,
    height: StaticValue<f64>
}

#[derive(Clone)]
pub struct UnpaddedBumper {
    algorithm: StaticValue<Arc<CollisionAlgorithm>>,
    algorithm_setter: DelayedSetter<'static,'static,Arc<CollisionAlgorithm>>,
    name: AllotmentName
}

impl UnpaddedBumper {
    pub fn new(name: &AllotmentName) -> UnpaddedBumper {
        let (algorithm_setter,algorithm) = delayed();
        let algorithm = derived(algorithm,|algorithm| algorithm.unwrap());
        UnpaddedBumper {
            algorithm, algorithm_setter,
            name: name.clone()
        }
    }
}

impl ContainerSpecifics for UnpaddedBumper {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, prep: &mut BoxPositionContext, children: &[(&Box<dyn Stackable>,BuildSize)]) -> StaticValue<f64> {
        /* build all_items, a solution-invariant structure of everything we need to bump each time */
        let mut items = vec![];
        for (_,size) in children {
            items.push(BumpItem {
                name: size.name.clone(),
                range: size.range.clone(),
                height: size.height.clone()
            });
            //prep.bump_requests.add(&size.name,&size.range,&size.height);
        }
        let all_items = Arc::new(items);
        /* Get the bumper */
        let algorithm = prep.bumper_factory.get(&self.name);
        /* Create a piece which can bump everything in all_items each time and yield a CollisionAlgorithm */
        let all_items2 = all_items.clone();
        let solved = short_memoized_arc(variable(move |answer_index| {
            let algorithm = algorithm.call(answer_index);
            for item in all_items2.iter() {
                algorithm.add_entry(&item.name,&item.range.call(answer_index),item.height.call(answer_index));
            }
            algorithm.bump();
            algorithm
        }));
        self.algorithm_setter.set(solved.clone());
        /* Cause algorithm to report how high we are per solution */
        derived(solved,|solved| solved.height())
    }

    fn set_locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for child in children.iter_mut() {
            /* Retrieve algorithm offset from bumper top */
            let name = child.name().clone();
            let offset = derived(self.algorithm.clone(),move |algorithm|
                algorithm.get(&name)
            );
            /* Add that to our reported top */
            child.locate(prep,&compose(top.clone(),offset,|a,b| a+b));
        }
    }
}
