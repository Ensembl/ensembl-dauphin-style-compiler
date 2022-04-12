use std::{marker::PhantomData, sync::{Weak, Arc}};

use super::{answer::Answer, store::Store};

pub(super) struct ShortStore<'a,T: 'a> {
    answers: Vec<Weak<T>>,
    phantom: PhantomData<&'a ()>
}

impl<'a,T: 'a> ShortStore<'a,T> {
    pub(super) fn new() -> ShortStore<'a,T> {
        ShortStore {
            answers: vec![],
            phantom: PhantomData
        }
    }
}

impl<'a,T: 'a> Store<'a,T> for ShortStore<'a,T> {
    fn set(&mut self, answer_index: &Answer<'a>, value: Arc<T>) {
        let index = answer_index.index();
        if self.answers.len() <= index {
            self.answers.resize(index+1,Weak::new());
        }
        self.answers[index] = Arc::downgrade(&value);
        answer_index.retain(&value);
    }

    fn get(&self, answer_index: &Answer) -> Option<Arc<T>> {
        self.answers.get(answer_index.index()).map(|x| x.upgrade()).flatten()
    }
}

#[cfg(test)]
mod test{
    use std::sync::{Arc, Weak};

    use crate::puzzle::AnswerAllocator;
    use crate::puzzle::store::Store;
    use super::ShortStore;

    #[test]
    fn short_smoke() {
        let mut a = AnswerAllocator::new();
        let mut ss = ShortStore::new();
        let a0 = a.get();
        let a1 = a.get();
        let a2 = a.get();
        assert_eq!(0,a0.index());
        assert_eq!(1,a1.index());
        assert_eq!(2,a2.index());
        ss.set(&a0,Arc::new(100));
        ss.set(&a2,Arc::new(102));
        assert_eq!(3,ss.answers.len());
        assert_eq!(Some(100),Weak::upgrade(&ss.answers[0]).map(|x| *x));
        assert_eq!(None,Weak::upgrade(&ss.answers[1]).map(|x| *x));
        assert_eq!(Some(102),Weak::upgrade(&ss.answers[2]).map(|x| *x));
        drop(a0);
        assert_eq!(None,Weak::upgrade(&ss.answers[0]).map(|x| *x));
        assert_eq!(None,Weak::upgrade(&ss.answers[1]).map(|x| *x));
        assert_eq!(Some(102),Weak::upgrade(&ss.answers[2]).map(|x| *x));
        let a0b = a.get();
        assert_eq!(0,a0b.index());
        ss.set(&a0b,Arc::new(103));
        assert_eq!(Some(103),Weak::upgrade(&ss.answers[0]).map(|x| *x));
        assert_eq!(None,Weak::upgrade(&ss.answers[1]).map(|x| *x));
        assert_eq!(Some(102),Weak::upgrade(&ss.answers[2]).map(|x| *x));
        drop(a1);
        drop(a2);
        assert_eq!(3,ss.answers.len());
    }
}
