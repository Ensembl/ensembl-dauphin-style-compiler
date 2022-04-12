use std::{sync::{Arc, Mutex}};

use crate::lock;

use super::{answer::{Answer}, value::{Value}, store::Store, short::ShortStore};

#[derive(Clone)]
pub struct UnknownSetter<'a,T: 'a>(Arc<Mutex<Box<dyn Store<'a,T> + 'a>>>);

pub type StaticUnknownSetter<T> = UnknownSetter<'static,T>;

impl<'a,T> UnknownSetter<'a,T> {
    pub fn set(&mut self, index: &mut Answer<'a>, value: T) {
        lock!(self.0).set(index,Arc::new(value));
    }
}

pub fn unknown<'f,'a,S,T: 'a>(store: S) -> (UnknownSetter<'a,T>,Value<'f,'a,Option<Arc<T>>>) where S: Store<'a,T> + 'a {
    let answers = Arc::new(Mutex::new(Box::new(store) as Box<dyn Store<T>>));
    let answers2 = answers.clone();
    let setter = UnknownSetter(answers);
    (setter,Value::new(move |answer_index| {
        answer_index.as_ref().map(|ai| lock!(answers2).get(ai))
    }))
}

pub fn unknown_function<'f,'a,S,T: 'a>(store: S) -> (UnknownSetter<'a,Value<'f,'a,T>>,Value<'f,'a,Option<T>>)
        where S: Store<'a,Value<'f,'a,T>> + 'a {
    let answers = Arc::new(Mutex::new(Box::new(store) as Box<dyn Store<Value<'f,'a,T>>>));
    let answers2 = answers.clone();
    let setter = UnknownSetter(answers);
    (setter,Value::new(move |answer_index| {
        if answer_index.is_none() { return None; }
        Some(lock!(answers2).get(answer_index.unwrap()).and_then(|v| v.inner(answer_index)))
    }))
}

pub fn short_unknown<'f,'a,T: 'a>() -> (UnknownSetter<'a,T>,Value<'f,'a,Option<Arc<T>>>) {
    unknown(ShortStore::new())
}

pub fn short_unknown_function<'f,'a,T: 'a>() -> (UnknownSetter<'a,Value<'f,'a,T>>,Value<'f,'a,Option<T>>) {
    unknown_function(ShortStore::new())
}

pub fn short_unknown_promise_clonable<'f,'a,T: Clone+'a>() -> (UnknownSetter<'a,T>,Value<'f,'a,T>) {
    let (setter,solver) = unknown(ShortStore::new());
    (setter,solver.unwrap().dearc())
}

pub fn short_unknown_function_promise<'f,'a,T: 'a>() -> (UnknownSetter<'a,Value<'f,'a,T>>,Value<'f,'a,T>) {
    let (setter,solver) = unknown_function(ShortStore::new());
    (setter,solver.unwrap())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use crate::puzzle::{AnswerAllocator, unknown::short_unknown_function, constant, short_unknown_promise_clonable};
    use super::short_unknown;

    #[test]
    fn unknown_smoke() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let mut a2 = a.get();
        let (mut us,u) = short_unknown();
        assert_eq!(None,u.call(&mut a.get()));
        us.set(&mut a1,9);
        us.set(&mut a2,25);
        assert_eq!(Some(Arc::new(9)),u.call(&a1)); 
        assert_eq!(Some(Arc::new(25)),u.call(&a2));
        assert_eq!(None,u.constant());
    }

    #[test]
    fn unknown_function() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let mut a2 = a.get();
        let (mut vs,v) = short_unknown_promise_clonable();
        vs.set(&mut a1,9);
        vs.set(&mut a2,5);
        let (mut us,u) = short_unknown_function();
        assert_eq!(None,u.call(&mut a.get()));
        us.set(&mut a1,constant(2));
        us.set(&mut a2,v);
        assert_eq!(Some(2),u.call(&a1));
        assert_eq!(Some(5),u.call(&a2));
        assert_eq!(None,u.constant());
    }
}
