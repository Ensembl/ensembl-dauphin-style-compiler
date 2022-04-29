use std::sync::{Arc, Mutex};
use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer}, lock};
use crate::{allotment::{style::allotmentname::AllotmentName, collision::{collisionalgorithm2::{BumpRequestSet, BumpResponses}}}};
use super::{globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec}, trainpersistent::TrainPersistent};

pub struct LocalBumpBuilder {
    builder: LocalValueBuilder<AllotmentName,Arc<BumpRequestSet>,BumpResponses>
}

impl LocalBumpBuilder {
    pub(crate) fn new() -> LocalBumpBuilder {
        LocalBumpBuilder {
            builder: LocalValueBuilder::new()
        }
    }

    pub(crate) fn set(&mut self, name: &AllotmentName, requests: &StaticValue<Arc<BumpRequestSet>>) {
        self.builder.entry(name.clone()).add_local(requests.clone());
    }

    pub(crate) fn global(&mut self, name: &AllotmentName) -> &StaticValue<BumpResponses> {
        self.builder.entry(name.clone()).get_global()
    }
}

pub struct LocalBump(LocalValueSpec<AllotmentName,Arc<BumpRequestSet>,BumpResponses>);

impl LocalBump {
    pub(crate) fn new(builder: &LocalBumpBuilder) -> LocalBump {
        LocalBump(LocalValueSpec::new(&builder.builder,|x| {
            // Multiple allotments with same name should be impossible
            x[0].clone()
        }))
    }

    pub(crate) fn add(&self, global: &mut GlobalBumpBuilder) {
        global.0.add(&self.0);
    }
}

pub struct GlobalBumpBuilder(GlobalValueBuilder<AllotmentName,Arc<BumpRequestSet>,BumpResponses>);

impl GlobalBumpBuilder {
    pub(crate) fn new() -> GlobalBumpBuilder {
        GlobalBumpBuilder(GlobalValueBuilder::new())
    }
}

#[derive(PartialEq,Eq,Hash)]
pub struct GlobalBump(GlobalValueSpec<AllotmentName,BumpResponses>);

impl GlobalBump {
    pub(crate) fn new(builder: GlobalBumpBuilder, answer: &mut StaticAnswer, persistent: &Arc<Mutex<TrainPersistent>>) -> GlobalBump {
        let persistent = persistent.clone();
        GlobalBump(GlobalValueSpec::new(builder.0,move |name,requests,answer| {
            let requests = requests.iter().map(|x| x.call(answer)).collect::<Vec<_>>();
            let mut persistent = lock!(persistent);
            persistent.bump_mut(name).make(&requests)
        },answer))
    }
}
