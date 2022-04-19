use std::{collections::{HashMap, hash_map::{DefaultHasher}}, sync::{Arc, Mutex}, fmt, hash::{Hash, Hasher}};

use peregrine_toolkit::{error, puzzle::{DelayedCommuteBuilder, UnknownSetter, StaticValue, StaticAnswer, short_unknown_promise_clonable, Answer, cache_constant, commute, short_memoized, commute_arc, short_memoized_arc, cache_constant_arc, constant, short_unknown_function, short_unknown_function_promise}, lock};

use crate::allotment::{style::allotmentname::AllotmentName};

use super::globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec};

pub struct LocalHeightTrackerBuilder(LocalValueBuilder<AllotmentName,f64>);

impl LocalHeightTrackerBuilder {
    pub(crate) fn new() -> LocalHeightTrackerBuilder {
        LocalHeightTrackerBuilder(LocalValueBuilder::new())
    }

    pub(crate) fn set(&mut self, name: &AllotmentName, value: StaticValue<f64>, independent_answer: &mut StaticAnswer) {
        let entry = self.0.entry(name.clone());
        entry.set_global(independent_answer,value.clone());
        entry.add_local(value);
    }

    pub(crate) fn global(&mut self, name: &AllotmentName) -> &StaticValue<f64> {
        self.0.entry(name.clone()).get_global()
    }
}

pub struct LocalHeightTracker(LocalValueSpec<AllotmentName,f64>);

impl LocalHeightTracker {
    pub(crate) fn new(builder: &LocalHeightTrackerBuilder, independent_answer: &mut StaticAnswer) -> LocalHeightTracker {
        LocalHeightTracker(LocalValueSpec::new(&builder.0,|x| {
            commute(x,0.,|x,y| x.max(*y)).dearc()
        },independent_answer))
    }

    pub(crate) fn add(&self, global: &mut GlobalHeightTrackerBuilder) {
        global.0.add(&self.0);
    }
}

pub struct GlobalHeightTrackerBuilder(GlobalValueBuilder<AllotmentName,f64>);

impl GlobalHeightTrackerBuilder {
    pub(crate) fn new() -> GlobalHeightTrackerBuilder {
        GlobalHeightTrackerBuilder(GlobalValueBuilder::new())
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(PartialEq,Eq,Hash)]
pub struct GlobalHeightTracker(GlobalValueSpec<AllotmentName,f64>);

impl GlobalHeightTracker {
    pub(crate) fn new(builder: GlobalHeightTrackerBuilder, answer: &mut StaticAnswer) -> GlobalHeightTracker {
        GlobalHeightTracker(GlobalValueSpec::new(builder.0,|x| {
            let v = x.iter().map(|x| **x).fold(0.,f64::max);
            (v,(v*100000.).round() as i64)
        },answer))
    }
}
