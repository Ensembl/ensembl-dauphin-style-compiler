use std::{rc::Rc};
use peregrine_toolkit::{puzzle::{derived, DelayedSetter, compose, StaticValue, promise_delayed, cache_constant_rc, short_memoized_rc, compose_slice_vec }};
use crate::{allotment::{core::{allotmentname::{AllotmentName}, boxpositioncontext::BoxPositionContext}, collision::{collisionalgorithm::{BumpRequestSet, BumpRequest, BumpResponses}}, layout::stylebuilder::{ContainerOrLeaf, BuildSize}}};
use super::container::ContainerSpecifics;

#[derive(Clone)]
pub struct Bumper {
    name: AllotmentName,
    results: StaticValue<BumpResponses>,
    results_setter: DelayedSetter<'static,'static,BumpResponses>
}

impl Bumper {
    pub fn new(name: &AllotmentName) -> Bumper {
        let (results_setter,results) = promise_delayed();
        Bumper {
            name: name.clone(),
            results, results_setter
        }
    }
}

impl ContainerSpecifics for Bumper {
    fn build_reduce(&self, prep: &mut BoxPositionContext, children: &[(&Box<dyn ContainerOrLeaf>,BuildSize)]) -> StaticValue<f64> {
        /* build all_items, a solution-invariant structure of everything we need to bump each time */
        let mut items = vec![];
        for (_,size) in children {
            items.push(size.to_value());
        }
        let items = compose_slice_vec(&items);
        /* build the ConcreteRequests for this container */
        let factory = prep.bumper_factory.clone();
        let concrete_req = derived(items,move |items| {
            let mut builder = factory.builder();
            for (name,height,range) in &items {
                builder.add(BumpRequest::new(name,range,*height));
            }
            Rc::new(BumpRequestSet::new(builder))
        });
        let concrete_req = cache_constant_rc(short_memoized_rc(concrete_req));
        prep.state_request.bump_mut().set(&self.name,&concrete_req);
        self.results_setter.set(prep.state_request.bump_mut().global(&self.name).clone());
        derived(self.results.clone(),|c| c.height() as f64)
    }

    fn set_locate(&self, prep: &mut BoxPositionContext, top: &StaticValue<f64>, children: &mut [&mut Box<dyn ContainerOrLeaf>]) {
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
