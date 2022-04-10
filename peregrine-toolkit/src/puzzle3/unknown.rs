use std::{sync::{Arc, Mutex}};

use crate::lock;

use super::{answer::{AnswerIndex}, solver::{Solver}, store::Store, short::AnswerStore};

#[derive(Clone)]
pub struct AnswerSetter<'a,T: 'a>(Arc<Mutex<Box<dyn Store<'a,T> + 'a>>>);

impl<'a,T> AnswerSetter<'a,T> {
    pub fn set(&mut self, index: &mut AnswerIndex<'a>, value: T) {
        lock!(self.0).set(index,Arc::new(value));
    }
}

pub fn unknown<'f,'a,S,T: 'a>(store: S) -> (AnswerSetter<'a,T>,Solver<'f,'a,Option<Arc<T>>>) where S: Store<'a,T> + 'a {
    let answers = Arc::new(Mutex::new(Box::new(store) as Box<dyn Store<T>>));
    let answers2 = answers.clone();
    let setter = AnswerSetter(answers);
    (setter,Solver::new(move |answer_index| {
        answer_index.as_ref().map(|ai| lock!(answers2).get(ai))
    }))
}

pub fn short_unknown<'f,'a,T: 'a>() -> (AnswerSetter<'a,T>,Solver<'f,'a,Option<Arc<T>>>) {
    unknown(AnswerStore::new())
}

pub fn short_unknown_promise_clonable<'f,'a,T: Clone+'a>() -> (AnswerSetter<'a,T>,Solver<'f,'a,T>) {
    let (setter,solver) = unknown(AnswerStore::new());
    (setter,solver.unwrap().dearc())
}
