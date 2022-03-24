use std::{mem, sync::Arc};

use super::{PuzzlePiece, PuzzleValueHolder, PuzzleValue};

pub struct FoldValue<T: Clone+'static>  {
    callback: Arc<dyn Fn(T,T) -> T>,
    output: PuzzlePiece<T>,
    inputs: Vec<PuzzleValueHolder<T>>
}

impl<T: Clone> FoldValue<T> {
    pub fn new<F>(output: PuzzlePiece<T>, cb: F) -> FoldValue<T> where F: Fn(T,T) -> T + 'static {
        FoldValue { 
            callback: Arc::new(cb),
            inputs: vec![],
            output
        }
    }

    pub fn add(&mut self, value: &PuzzleValueHolder<T>) {
        self.inputs.push(value.clone());
    }

    pub fn merge(&mut self, other: &FoldValue<T>) {
        self.inputs.extend(&mut other.inputs.iter().cloned());
    }

    pub fn build(&mut self) {
        let dependencies = self.inputs.iter().map(|holder| holder.dependency()).collect::<Vec<_>>();
        let inputs = mem::replace(&mut self.inputs,vec![]);
        let callback = self.callback.clone();
        self.output.add_solver(&dependencies, move |solution| {
            let values = inputs.iter().map(|piece| piece.get(solution).as_ref().clone());
            values.fold(None, |a: Option<T>, b| {
                Some(if let Some(a) = a { callback(a,b) } else { b })
            })
        });
    }

    pub fn get(&self) -> &PuzzlePiece<T> { &self.output }
}
