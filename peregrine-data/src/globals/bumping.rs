use std::{sync::{Arc, Mutex}, rc::Rc};
use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer}, lock, log};
use crate::{allotment::{collision::{bumprequest::BumpRequestSet, algorithmbuilder::BumpResponses}, core::allotmentname::AllotmentName}};
use super::{globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec}, trainpersistent::TrainPersistent};

pub struct LocalBumpBuilder {
    builder: LocalValueBuilder<AllotmentName,Rc<(BumpRequestSet,bool)>,BumpResponses>
}

impl LocalBumpBuilder {
    pub(crate) fn new() -> LocalBumpBuilder {
        LocalBumpBuilder {
            builder: LocalValueBuilder::new()
        }
    }

    pub(crate) fn set(&mut self, name: &AllotmentName, requests: &StaticValue<Rc<(BumpRequestSet,bool)>>) {
        self.builder.entry(name.clone()).add_local(requests.clone());
    }

    pub(crate) fn global(&mut self, name: &AllotmentName) -> &StaticValue<BumpResponses> {
        self.builder.entry(name.clone()).get_global()
    }
}

pub struct LocalBump(LocalValueSpec<AllotmentName,Rc<(BumpRequestSet,bool)>,BumpResponses>);

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

pub struct GlobalBumpBuilder(GlobalValueBuilder<AllotmentName,Rc<(BumpRequestSet,bool)>,BumpResponses>);

impl GlobalBumpBuilder {
    pub(crate) fn new() -> GlobalBumpBuilder {
        GlobalBumpBuilder(GlobalValueBuilder::new(true))
    }
}

#[derive(PartialEq,Eq,Hash)]
pub struct GlobalBump(GlobalValueSpec<AllotmentName,BumpResponses>);

impl GlobalBump {
    pub(crate) fn new(builder: GlobalBumpBuilder, answer: &mut StaticAnswer, persistent: &Arc<Mutex<TrainPersistent>>) -> GlobalBump {
        let persistent = persistent.clone();
        GlobalBump(GlobalValueSpec::new(builder.0,move |name,requests,answer| {
            let mut requests = requests.iter().map(|x| x.call(answer)).collect::<Vec<_>>();
            let mut persistent = lock!(persistent);
            let use_wall = requests.iter().any(|x| x.1);
            let requests = requests.drain(..).map(|x| x.0.clone()).collect::<Vec<_>>();
            persistent.bump_mut(name,use_wall).make(&requests)
        },answer))
    }

    pub(crate) fn add(&self, local: &LocalBump, answer: &mut StaticAnswer) {
        self.0.add(&local.0,answer);
    }
}
