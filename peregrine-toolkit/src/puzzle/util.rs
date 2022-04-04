use std::{mem, sync::Arc};

use crate::{puzzle::{DerivedPuzzlePiece}, log};

use super::{PuzzlePiece, PuzzleValueHolder, PuzzleValue, PuzzleBuilder, ConstantPuzzlePiece, DelayedPuzzleValue};

pub struct FoldValue<T: Clone+'static>  {
    callback: Arc<dyn Fn(T,T) -> T>,
    output: DelayedPuzzleValue<T>,
    inputs: Vec<PuzzleValueHolder<T>>
}

impl<T: Clone> FoldValue<T> {
    pub fn new<F>(output: DelayedPuzzleValue<T>, cb: F) -> FoldValue<T> where F: Fn(T,T) -> T + 'static {
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

    pub fn build(&mut self, builder: &mut PuzzleBuilder, initial: T) {
        let dependencies = self.inputs.iter().map(|holder| holder.dependency()).collect::<Vec<_>>();
        let inputs = mem::replace(&mut self.inputs,vec![]);
        let callback = self.callback.clone();
        let answer = builder.new_combination(&dependencies, move |solution| {
            let initial = initial.clone();
            let values = inputs.iter().map(move |piece| piece.get(solution).as_ref().clone());
            values.fold(initial, |a, b| {
                callback(a,b)
            })
        });
        self.output.set(builder,PuzzleValueHolder::new(answer));
    }

    pub fn get(&self) -> PuzzleValueHolder<T> { PuzzleValueHolder::new(self.output.clone()) }
}

pub struct CommutingSequence<T: Clone+'static> {
    default: Arc<T>,
    callback: Arc<dyn Fn(&T,&T) -> T>,
    inputs: Vec<PuzzleValueHolder<T>>
}

impl<T: Clone+'static> CommutingSequence<T> {
    pub fn new<F>(default: T, cb: F) -> CommutingSequence<T> where F: Fn(&T,&T) -> T + 'static {
        CommutingSequence {
            default: Arc::new(default),
            callback: Arc::new(cb),
            inputs: vec![]
        }
    }

    pub fn add(&mut self, value: &PuzzleValueHolder<T>) {
        self.inputs.push(value.clone());
    }

    pub fn merge(&mut self, other: &CommutingSequence<T>) {
        self.inputs.extend(&mut other.inputs.iter().cloned());
    }

    pub fn build(&mut self, builder: &PuzzleBuilder) -> PuzzleValueHolder<T> {
        /* Examine what we know are constants */
        let mut constant_acc : Option<Arc<T>> = None;
        let mut variable = vec![];
        for v in &self.inputs {
            match v.known_constant_value() {
                Some(new_constant) => {
                    /* This is a constant */
                    constant_acc = Some(if let Some(old_value) = &constant_acc {
                        /* Merge with existing constant */
                        Arc::new((self.callback)(old_value,&new_constant))
                    } else {
                        /* First constant */
                       new_constant
                    });
                },
                None => {
                    /* This is a variable */
                    variable.push(v);
                }
            }
        }
        /* Build something! */
        if variable.len() > 1 {
            /* More than onevariable: a full unknown PuzzlePiece is required. Boo! */
            let dependencies = variable.iter().map(|x| x.dependency()).collect::<Vec<_>>();
            let mut variables = mem::replace(&mut self.inputs, vec![]);
            let last = variables.pop().unwrap(); // we know len()>1, so safe
            let callback = self.callback.clone();
            let piece = builder.new_combination(&dependencies, move |solution| {
                let mut value = last.get(solution).as_ref().clone();
                for v in &variables {
                    /* add in each variable */
                    value = (callback)(&value,&v.get(solution));
                }
                if let Some(constant) = &constant_acc {
                    /* add in constant, if present */
                    value = (callback)(&value,constant);
                }
                value
            });
            PuzzleValueHolder::new(piece)
        } else if variable.len() == 1 {
            /* One variable, we can just create a DerivedPiece */
            let callback = self.callback.clone();
            PuzzleValueHolder::new(DerivedPuzzlePiece::new(variable[0].clone(),move |value| {
                if let Some(constant) = &constant_acc {
                    /* add in constant */
                    (callback)(&value,constant)
                } else {
                    /* no constant */
                    value.clone()
                }
            }))
        } else if let Some(constant) = &constant_acc {
            /* No variables, just a constant */
            PuzzleValueHolder::new(ConstantPuzzlePiece::new(constant.as_ref().clone()))
        } else {
            /* No constant or variables */
            PuzzleValueHolder::new(ConstantPuzzlePiece::new(self.default.as_ref().clone()))
        }
    }
}

pub fn compose2<F,T,U,V>(builder: &PuzzleBuilder, a: &PuzzleValueHolder<T>, b: &PuzzleValueHolder<U>, cb: F) -> PuzzleValueHolder<V> where F: Fn(&T,&U) -> V + 'static {
    match (a.known_constant_value(),b.known_constant_value()) {
        (Some(a),Some(b)) => PuzzleValueHolder::new(ConstantPuzzlePiece::new(cb(&a,&b))),
        (Some(a),None) => PuzzleValueHolder::new(DerivedPuzzlePiece::new(b.clone(),move |b| cb(&a,b))),
        (None,Some(b)) => PuzzleValueHolder::new(DerivedPuzzlePiece::new(a.clone(),move |a| cb(a,&b))),
        (None,None) => {
            let a2 = a.clone();
            let b2 = b.clone();
            let mut piece = builder.new_combination(&[a.dependency(),b.dependency()],  move |solution| {
                cb(&a2.get(solution),&b2.get(solution))
            });
            PuzzleValueHolder::new(piece)
        }
    }
}

pub fn build_puzzle_vec<T: Clone>(builder: &PuzzleBuilder, input: &[PuzzleValueHolder<T>]) -> PuzzleValueHolder<Vec<Arc<T>>> {
    let constants = input.iter()
        .filter_map(|x| x.known_constant_value()).collect::<Vec<_>>();
    if constants.len() == input.len() {
        return PuzzleValueHolder::new(ConstantPuzzlePiece::new(constants));
    }
    let dependencies = input.iter().map(|x| x.dependency()).collect::<Vec<_>>();
    let input = input.iter().cloned().collect::<Vec<_>>();
    let piece = builder.new_combination(&dependencies, move |solution| {
        input.iter().map(|x| x.get(solution)).collect::<Vec<_>>()
    });
    PuzzleValueHolder::new(piece)
}
