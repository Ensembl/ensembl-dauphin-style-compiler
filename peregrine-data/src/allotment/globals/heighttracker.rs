use std::hash::Hash;
use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer, commute}};
use crate::allotment::{style::allotmentname::AllotmentName};
use super::globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec};

pub struct LocalHeightTrackerBuilder(LocalValueBuilder<AllotmentName,f64,f64>);

impl LocalHeightTrackerBuilder {
    pub(crate) fn new() -> LocalHeightTrackerBuilder {
        LocalHeightTrackerBuilder(LocalValueBuilder::new())
    }

    pub(crate) fn set(&mut self, name: &AllotmentName, value: StaticValue<f64>) {
        let entry = self.0.entry(name.clone());
        entry.add_local(value);
    }

    pub(crate) fn global(&mut self, name: &AllotmentName) -> &StaticValue<f64> {
        self.0.entry(name.clone()).get_global()
    }
}

pub struct LocalHeightTracker(LocalValueSpec<AllotmentName,f64,f64>);

impl LocalHeightTracker {
    pub(crate) fn new(builder: &LocalHeightTrackerBuilder) -> LocalHeightTracker {
        LocalHeightTracker(LocalValueSpec::new(&builder.0,|x| {
            commute(x,0.,|x,y| x.max(*y)).dearc()
        }))
    }

    pub(crate) fn add(&self, global: &mut GlobalHeightTrackerBuilder) {
        global.0.add(&self.0);
    }
}

pub struct GlobalHeightTrackerBuilder(GlobalValueBuilder<AllotmentName,f64,f64>);

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
        GlobalHeightTracker(GlobalValueSpec::new(builder.0,|x,answer| {
            let v = x.iter().map(|x| x.call(&answer)).fold(0.,f64::max);
            (v,(v*100000.).round() as i64)
        },answer))
    }
}
