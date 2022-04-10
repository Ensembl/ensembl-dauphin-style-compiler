use std::sync::{Mutex, Arc};

use crate::lock;

use super::{value::{Value}, store::Store, short::AnswerStore};

fn do_memoized<'f,'a,S,T: 'a,U:'a,F: 'f>(input: Value<'f,'a,U>, store: S, arcify: F) -> Value<'f,'a,Arc<T>>
        where S: Store<'a,T> + 'a, F: Fn(U) -> Arc<T> + 'f {
    let arcify = Arc::new(arcify);
    let store_arc = Arc::new(Mutex::new(Box::new(store) as Box<dyn Store<'a,T> + 'a>));
    Value::new(move |answer_index| {
        let store = lock!(store_arc);
        if let Some(answer_index) = answer_index {
            if let Some(old) = store.get(answer_index) {
                Some(old.clone())
            } else{
                drop(store);
                let inner_value = arcify(input.call(answer_index));
                let mut store = lock!(store_arc);
                store.set(answer_index,inner_value.clone());
                Some(inner_value)
            }
        } else {
            input.inner(answer_index).map(|x| arcify(x))
        }
    })
}

pub fn memoized<'f,'a,S,T: 'a>(input: Value<'f,'a,T>, store: S) -> Value<'f,'a,Arc<T>> where S: Store<'a,T> + 'a {
    do_memoized(input,store,|x| Arc::new(x))
}

pub fn memoized_arc<'f,'a,S,T: 'a>(input: Value<'f,'a,Arc<T>>, store: S) -> Value<'f,'a,Arc<T>> where S: Store<'a,T> + 'a {
    do_memoized(input,store,|x| x)
}

pub fn short_memoized<'f,'a,T: 'a>(input: Value<'f,'a,T>) -> Value<'f,'a,Arc<T>> {
    memoized(input,AnswerStore::new())
}

pub fn short_memoized_arc<'f,'a,T: 'a>(input: Value<'f,'a,Arc<T>>) -> Value<'f,'a,Arc<T>> {
    memoized_arc(input,AnswerStore::new())
}

pub fn short_memoized_clonable<'f: 'a,'a,T: Clone+'a>(input: Value<'f,'a,T>) -> Value<'f,'a,T> {
    memoized(input,AnswerStore::new()).dearc()
}
