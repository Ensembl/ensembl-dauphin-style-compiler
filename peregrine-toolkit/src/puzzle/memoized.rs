use std::{rc::Rc, cell::RefCell};

use super::{value::{Value}, store::Store, short::ShortStore};

fn do_memoized<'f:'a,'a,S,T: 'a,U:'a,F: 'f>(input: Value<'f,'a,U>, store: S, rcify: F) -> Value<'f,'a,Rc<T>>
        where S: Store<'a,T> + 'a, F: Fn(U) -> Rc<T> + 'f {
    let rcify = Rc::new(rcify);
    let store_rc = Rc::new(RefCell::new(Box::new(store) as Box<dyn Store<'a,T> + 'a>));
    Value::new(move |answer_index| {
        let store = store_rc.borrow_mut();
        if let Some(answer_index) = answer_index {
            if let Some(old) = store.get(answer_index) {
                Some(old.clone())
            } else{
                drop(store);
                let inner_value = rcify(input.call(answer_index));
                let mut store = store_rc.borrow_mut();
                store.set(answer_index,inner_value.clone());
                Some(inner_value)
            }
        } else {
            input.inner(answer_index).map(|x| rcify(x))
        }
    })
}

pub fn memoized<'f:'a,'a,S,T: 'a>(input: Value<'f,'a,T>, store: S) -> Value<'f,'a,Rc<T>> where S: Store<'a,T> + 'a {
    do_memoized(input,store,|x| Rc::new(x))
}

pub fn memoized_rc<'f:'a,'a,S,T: 'a>(input: Value<'f,'a,Rc<T>>, store: S) -> Value<'f,'a,Rc<T>> where S: Store<'a,T> + 'a {
    do_memoized(input,store,|x| x)
}

pub fn short_memoized<'f:'a,'a,T: 'a>(input: Value<'f,'a,T>) -> Value<'f,'a,Rc<T>> {
    memoized(input,ShortStore::new())
}

pub fn short_memoized_rc<'f:'a,'a,T: 'a>(input: Value<'f,'a,Rc<T>>) -> Value<'f,'a,Rc<T>> {
    memoized_rc(input,ShortStore::new())
}

pub fn short_memoized_clonable<'f: 'a,'a,T: Clone+'a>(input: Value<'f,'a,T>) -> Value<'f,'a,T> {
    memoized(input,ShortStore::new()).derc()
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};

    use crate::{puzzle::{compose_slice, cache_constant, derived, constant, AnswerAllocator}};

    #[test]
    fn memoized_smoke() {
        for do_memoize in &[false,true] {
            let count = Rc::new(RefCell::new(0));
            /* consts will respond to None directly ... */
            let mut consts = vec![];
            for i in 0..10 {
                consts.push(constant(i));
            }
            /* ... but deriveds will not */
            let mut deriveds = vec![];
            for c in &consts {
                let c = c.clone();
                let count2 = count.clone();
                deriveds.push(derived(c,move |v| {
                    *count2.borrow_mut() += 1;
                    v*v
                }));
            }
            /* Let's build them into a single array and memoize that */
            let value = compose_slice(&deriveds, |v| v.to_vec());
            let value = if *do_memoize {
                cache_constant(value)
            } else {
                derived(value,|value| Rc::new(value))
            };
            /* Make a single sum value, for easy testing */
            let total = derived(value,move |value| {
                value.iter().fold(0,|a,b| a+*b)
            });
            /* Evaluate twice (to try to trick into calling deriveds twice: shouldn't as we have memoized) */
            let mut aia = AnswerAllocator::new();
            let mut ai1 = aia.get();
            let mut ai2 = aia.get();
            let v1 = total.call(&mut ai1);
            let v2 = total.call(&mut ai2);
            assert_eq!(285,v1); /* 1+4+...+64+81 */
            assert_eq!(285,v2);
            assert_eq!(if *do_memoize { 10 } else { 20 },*count.borrow());
        }
    }
}