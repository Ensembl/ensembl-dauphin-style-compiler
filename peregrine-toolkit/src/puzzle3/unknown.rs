use std::{sync::{Arc, Mutex}, marker::PhantomData};

use crate::lock;

use super::{answer::{AnswerIndex}, solver::{Solver}, store::Store, short::AnswerStore};

#[derive(Clone)]
pub struct AnswerSetter<'a,T: 'a>(Arc<Mutex<Box<dyn Store<'a,T> + 'a>>>,PhantomData<&'a T>);

impl<'a,T> AnswerSetter<'a,T> {
    pub(super) fn set(&mut self, index: &mut AnswerIndex<'a>, value: T) {
        lock!(self.0).set(index,Arc::new(value));
    }
}

pub fn unknown<'f,'a,S,T: 'a>(store: S) -> (AnswerSetter<'a,T>,Solver<'f,'a,Option<Arc<T>>>) where S: Store<'a,T> + 'a {
    let answers = Arc::new(Mutex::new(Box::new(store) as Box<dyn Store<T>>));
    let answers2 = answers.clone();
    let setter = AnswerSetter(answers,PhantomData);
    (setter,Solver::new(move |answer_index| {
        answer_index.as_mut().map(|ai| lock!(answers2).get(ai))
    }))
}

pub fn short_unknown<'f,'a,T: 'a>() -> (AnswerSetter<'a,T>,Solver<'f,'a,Option<Arc<T>>>) {
    unknown(AnswerStore::new())
}
