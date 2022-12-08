use std::sync::Arc;

use peregrine_data::{ProgramShapesBuilder, reactive::{self, Reactive}};
use crate::{Message};

#[derive(Clone)]
pub(crate) struct AreaVariables<'a> {
    tlbr: (reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>)
}

impl<'a> AreaVariables<'a> {
    pub(crate) fn new(reactive: &Reactive<'a>) -> AreaVariables<'a> {
        AreaVariables {
            tlbr: (reactive.variable(0.),reactive.variable(0.),reactive.variable(0.),reactive.variable(0.)),
        }
    }

    pub(crate) fn update(&mut self, tlbr: (f64,f64,f64,f64)) {
        self.tlbr.0.set(tlbr.0);
        self.tlbr.1.set(tlbr.1);
        self.tlbr.2.set(tlbr.2);
        self.tlbr.3.set(tlbr.3);
    }

    pub(crate) fn tlbr(&self) -> &(reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>) { &self.tlbr }
}

pub(crate) trait Spectre {
    fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message>;
}

impl Spectre for Arc<dyn Spectre> {
    fn draw(&self, shapes: &mut ProgramShapesBuilder) -> Result<(),Message> {
        self.as_ref().draw(shapes)
    }
}
