use std::sync::{Arc};

use peregrine_toolkit::{puzzle::{derived, DelayedSetter, compose, StaticValue, compose_slice, promise_delayed, cache_constant_arc, short_memoized_arc }};

use crate::{allotment::{core::{allotmentname::{AllotmentNamePart, AllotmentName}, boxtraits::{Stackable, BuildSize, ContainerSpecifics, Coordinated}, boxpositioncontext::BoxPositionContext}, style::{style::{ContainerAllotmentStyle}}, collision::{collisionalgorithm::{BumpRequestSet, BumpRequest, BumpResponses}}}, CoordinateSystem};

use super::{container::{Container}};

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
pub struct UnpaddedBumper {
    name: AllotmentName,
    results: StaticValue<BumpResponses>,
    results_setter: DelayedSetter<'static,'static,BumpResponses>
}

impl UnpaddedBumper {
    pub fn new(name: &AllotmentName) -> UnpaddedBumper {
        let (results_setter,results) = promise_delayed();
        UnpaddedBumper {
            name: name.clone(),
            results, results_setter
        }
    }
}

impl ContainerSpecifics for UnpaddedBumper {
    fn cloned(&self) -> Box<dyn ContainerSpecifics> { Box::new(self.clone()) }

    fn build_reduce(&mut self, prep: &mut BoxPositionContext, children: &[(&Box<dyn Stackable>,BuildSize)]) -> StaticValue<f64> {
        /* build all_items, a solution-invariant structure of everything we need to bump each time */
        let mut items = vec![];
        for (_,size) in children {
            items.push(size.to_value());
        }
        let items = compose_slice(&items,|x| x.to_vec());
        /* build the ConcreteRequests for this container */
        let factory = prep.bumper_factory.clone();
        let concrete_req = derived(items,move |items| {
            let mut builder = factory.builder();
            for (name,height,range) in &items {
                builder.add(BumpRequest::new(name,range,*height));
            }
            Arc::new(BumpRequestSet::new(builder))
        });
        let concrete_req = cache_constant_arc(short_memoized_arc(concrete_req));
        prep.state_request.bump_mut().set(&self.name,&concrete_req);
        self.results_setter.set(prep.state_request.bump_mut().global(&self.name).clone());
        derived(self.results.clone(),|c| c.height() as f64)
    }

    fn set_locate(&mut self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn Stackable>]) {
        for child in children.iter_mut() {
            /* Retrieve algorithm offset from bumper top */
            let name = child.name().clone();
            let offset = derived(self.results.clone(),move |algorithm|
                algorithm.get(&name) as f64
            );
            /* Add that to our reported top */
            child.locate(prep,&compose(top.clone(),offset,|a,b| a+b));
        }
    }
}
