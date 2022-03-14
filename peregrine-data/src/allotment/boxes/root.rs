use std::{sync::{Arc, Mutex}, mem, cmp::max};

use peregrine_toolkit::{puzzle::{ConstantPuzzlePiece, PuzzleValueHolder, PuzzleBuilder, PuzzlePiece, PuzzleValue, PuzzleSolution}, lock};

use crate::{CoordinateSystemVariety, CoordinateSystem};

use super::boxtraits::Stackable;

struct MaxValue {
    output: PuzzlePiece<f64>,
    inputs: Vec<PuzzleValueHolder<f64>>
}

impl MaxValue {
    fn new(builder: &PuzzleBuilder, default: f64) -> MaxValue {
        MaxValue { 
            inputs: vec![],
            output: builder.new_piece(Some(default))
        }
    }

    fn add(&mut self, value: &PuzzleValueHolder<f64>) {
        self.inputs.push(value.clone());
    }

    fn build(&mut self) {
        let dependencies = self.inputs.iter().map(|holder| holder.dependency()).collect::<Vec<_>>();
        let inputs = mem::replace(&mut self.inputs,vec![]);
        self.output.add_solver(&dependencies, move |solution| {
            let values = inputs.iter().map(|piece| piece.get(solution).as_ref().clone());
            values.fold(None, |a: Option<f64>, b| {
                Some(if let Some(a) = a { a.max(b) } else { b })
            })
        });
    }

    fn get(&self) -> &PuzzlePiece<f64> { &self.output }
}

#[derive(PartialEq,Clone,Debug)]
pub struct PlayingField2 {
    height: f64,
    squeeze: (f64,f64),
}

impl PlayingField2 {
    pub fn empty() -> PlayingField2 {
        PlayingField2 {
            height: 0.,
            squeeze: (0.,0.)
        }
    }

    pub fn union(&self, other: &PlayingField2) -> PlayingField2 {
        PlayingField2 {
            height: self.height.max(other.height),
            squeeze: (self.squeeze.0.max(other.squeeze.0),self.squeeze.1.max(other.squeeze.1))
        }
    }

    pub fn height(&self) -> f64 { self.height }
    pub fn squeeze(&self) -> (f64,f64) { self.squeeze }
}

pub struct PlayingFieldHolder {
    top: MaxValue,
    bottom: MaxValue,
    left: MaxValue,
    right: MaxValue
}

impl PlayingFieldHolder {
    fn new(puzzle: &mut PuzzleBuilder) -> PlayingFieldHolder {
        PlayingFieldHolder {
            top: MaxValue::new(puzzle,0.),
            bottom: MaxValue::new(puzzle,0.),
            left: MaxValue::new(puzzle,0.),
            right: MaxValue::new(puzzle,0.),
        }
    }

    fn get(&self, solution: &PuzzleSolution) -> PlayingField2 {
        PlayingField2 {
            height: *self.top.get().get(solution) + *self.bottom.get().get(solution),
            squeeze: (*self.left.get().get(solution),*self.right.get().get(solution))
        }
    }

    fn set(&mut self, coord_system: &CoordinateSystem, value: &PuzzleValueHolder<f64>) {
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

    fn ready(&mut self) {
        self.top.build();
        self.bottom.build();
        self.left.build();
        self.right.build();
    }
}

#[derive(Clone)]
pub struct Root {
    playing_field: Arc<Mutex<PlayingFieldHolder>>
}

impl Root {
    pub fn new(puzzle: &mut PuzzleBuilder) -> Root { 
        let playing_field = Arc::new(Mutex::new(PlayingFieldHolder::new(puzzle)));
        let playing_field2 = playing_field.clone();
        puzzle.add_ready(move |_| { lock!(playing_field2).ready(); });
        Root { playing_field }
    }

    pub fn add_child(&self, child: &dyn Stackable) {
        child.set_indent(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        child.set_top(&PuzzleValueHolder::new(ConstantPuzzlePiece::new(0.)));
        lock!(self.playing_field).set(child.coordinate_system(),&child.height());
    }

    pub fn playing_field(&self, solution: &PuzzleSolution) -> PlayingField2 {
        lock!(self.playing_field).get(solution)
    }
}
