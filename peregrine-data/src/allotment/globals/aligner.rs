use peregrine_toolkit::{puzzle::{StaticValue, StaticAnswer, commute}};
use super::globalvalue::{LocalValueBuilder, LocalValueSpec, GlobalValueBuilder, GlobalValueSpec};

pub struct LocalAlignerBuilder(LocalValueBuilder<String,f64,f64>);

impl LocalAlignerBuilder {
    pub(crate) fn new() -> LocalAlignerBuilder {
        LocalAlignerBuilder(LocalValueBuilder::new())
    }

    pub(crate) fn set(&mut self, name: &String, value: StaticValue<f64>) {
        self.0.entry(name.clone()).add_local(value);
    }

    pub(crate) fn global(&mut self, name: &String) -> &StaticValue<f64> {
        self.0.entry(name.clone()).get_global()
    }
}

pub struct LocalAligner(LocalValueSpec<String,f64,f64>);

impl LocalAligner {
    pub(crate) fn new(builder: &LocalAlignerBuilder) -> LocalAligner {
        LocalAligner(LocalValueSpec::new(&builder.0,|x| {
            commute(x,0.,|x,y| x.max(*y)).dearc()
        }))
    }

    pub(crate) fn add(&self, global: &mut GlobalAlignerBuilder) {
        global.0.add(&self.0);
    }
}

pub struct GlobalAlignerBuilder(GlobalValueBuilder<String,f64,f64>);

impl GlobalAlignerBuilder {
    pub(crate) fn new() -> GlobalAlignerBuilder {
        GlobalAlignerBuilder(GlobalValueBuilder::new())
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(PartialEq,Eq,Hash)]
pub struct GlobalAligner(GlobalValueSpec<String,f64>);

impl GlobalAligner {
    pub(crate) fn new(builder: GlobalAlignerBuilder, answer: &mut StaticAnswer) -> GlobalAligner {
        GlobalAligner(GlobalValueSpec::new(builder.0,|_,x,answer| {
            let v = x.iter().map(|x| x.call(&answer)).fold(f64::NEG_INFINITY,f64::max);
            (v,(v*100000.).round() as i64)
        },answer))
    }

    pub(crate) fn add(&self, local: &LocalAligner, answer: &mut StaticAnswer) {
        self.0.add(&local.0,answer);
    }
}
