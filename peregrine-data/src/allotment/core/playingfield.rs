use peregrine_toolkit::{puzzle::{DelayedCommuteBuilder, StaticValue, StaticAnswer}};

use crate::{CoordinateSystemVariety, CoordinateSystem};

#[derive(PartialEq,Clone,Debug)]
pub struct PlayingField {
    height: f64,
    squeeze: (f64,f64),
}

impl PlayingField {
    pub fn empty() -> PlayingField {
        PlayingField {
            height: 0.,
            squeeze: (0.,0.)
        }
    }

    pub fn union(&self, other: &PlayingField) -> PlayingField {
        PlayingField {
            height: self.height.max(other.height),
            squeeze: (self.squeeze.0.max(other.squeeze.0),self.squeeze.1.max(other.squeeze.1))
        }
    }

    pub fn height(&self) -> f64 { self.height }
    pub fn squeeze(&self) -> (f64,f64) { self.squeeze }
}

fn new_max() -> DelayedCommuteBuilder<'static,f64> {
    DelayedCommuteBuilder::new(|a: &f64,b| a.max(*b))
}

pub struct PlayingFieldHolder {
    top: DelayedCommuteBuilder<'static,f64>,
    bottom: DelayedCommuteBuilder<'static,f64>,
    left: DelayedCommuteBuilder<'static,f64>,
    right: DelayedCommuteBuilder<'static,f64>
}

#[derive(Clone)]
pub struct PlayingFieldPieces {
    pub top: StaticValue<f64>,
    pub bottom: StaticValue<f64>,
    pub left: StaticValue<f64>,
    pub right: StaticValue<f64>
}

impl PlayingFieldPieces {
    pub(crate) fn new(holder: &PlayingFieldHolder) -> PlayingFieldPieces {
        PlayingFieldPieces {
            top: holder.top.solver().clone().dearc(),
            bottom:holder.bottom.solver().clone().dearc(),
            left: holder.left.solver().clone().dearc(),
            right: holder.right.solver().clone().dearc()
        }
    }
}

impl PlayingFieldHolder {
    pub(crate) fn new() -> PlayingFieldHolder {
        PlayingFieldHolder {
            top: new_max(),
            bottom: new_max(),
            left: new_max(),
            right: new_max(),
        }
    }

    pub(crate) fn get(&self, answer_index: &mut StaticAnswer) -> PlayingField {
        PlayingField {
            height: *self.top.solver().call(answer_index) + *self.bottom.solver().call(answer_index),
            squeeze: (*self.left.solver().call(answer_index),*self.right.solver().call(answer_index))
        }
    }

    pub(crate) fn set(&mut self, coord_system: &CoordinateSystem, value: &StaticValue<f64>) {
        let var = match (&coord_system.0,coord_system.1) {
            (CoordinateSystemVariety::Tracking, false) => &mut self.top,
            (CoordinateSystemVariety::Tracking, true) => &mut self.bottom,
            (CoordinateSystemVariety::TrackingWindow, false) => &mut self.top,
            (CoordinateSystemVariety::TrackingWindow, true) => &mut self.bottom,
            (CoordinateSystemVariety::Sideways, false) => &mut self.left,
            (CoordinateSystemVariety::Sideways, true) => &mut self.right,
            _ => { return; }
        };
        var.add(value);
    }

    pub(crate) fn ready(&mut self) {
        self.top.build(0.);
        self.bottom.build(0.);
        self.left.build(0.);
        self.right.build(0.);
    }
}
