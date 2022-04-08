use std::{marker::PhantomData, sync::{Weak, Arc}};

use super::{answer::AnswerIndex, store::Store};

pub(super) struct AnswerStore<'a,T: 'a> {
    answers: Vec<Weak<T>>,
    phantom: PhantomData<&'a ()>
}

impl<'a,T: 'a> AnswerStore<'a,T> {
    pub(super) fn new() -> AnswerStore<'a,T> {
        AnswerStore {
            answers: vec![],
            phantom: PhantomData
        }
    }
}

impl<'a,T: 'a> Store<'a,T> for AnswerStore<'a,T> {
    fn set(&mut self, answer_index: &mut AnswerIndex<'a>, value: Arc<T>) {
        let index = answer_index.index();
        if self.answers.len() <= index {
            self.answers.resize(index+1,Weak::new());
        }
        self.answers[index] = Arc::downgrade(&value);
        answer_index.retain(&value);
    }

    fn get(&self, answer_index: &AnswerIndex) -> Option<Arc<T>> {
        self.answers.get(answer_index.index()).map(|x| x.upgrade()).flatten()
    }
}
