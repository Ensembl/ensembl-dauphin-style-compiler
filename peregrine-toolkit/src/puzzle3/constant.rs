use std::{sync::{Arc, Mutex}};

use crate::lock;

use super::value::{Value};

pub fn constant<'f,'a,T: Clone + 'a>(value: T) -> Value<'f,'a,T> {
    Value::new(move |_| Some(value.clone()))
}

enum ConstantResult<T> {
    NotTried,
    NotAvailable,
    Available(Arc<T>)
}

impl<T> ConstantResult<T> {
    fn try_constant<'f,'a,F,U>(&mut self, input: &Value<'f,'a,U>, arcify: F) -> Option<Arc<T>>  where F: Fn(U) -> Arc<T> + 'f {
        if let Some(value) = input.constant() {
            let value = arcify(value);
            *self = ConstantResult::Available(value.clone());
            Some(value)
        } else {
            *self = ConstantResult::NotAvailable;
            None
        }

    }
}

fn do_cache_constant<'a,'f, T: 'a,U: 'a, F: 'f>(input: Value<'f,'a,U>, arcify: F) -> Value<'f,'a,Arc<T>>
        where F: Fn(U) -> Arc<T> + 'f {
    let constant = Arc::new(Mutex::new(ConstantResult::NotTried));
    let arcify = Arc::new(arcify);
    Value::new(move |answer_index| {
        let mut cache = lock!(constant);
        if let ConstantResult::NotTried = &*cache {
            cache.try_constant(&input,&*arcify);
        }
        match &*cache {
            ConstantResult::NotAvailable => {
                input.inner(answer_index).map(|x| arcify(x))
            },
            ConstantResult::Available(v) => Some(v.clone()),
            ConstantResult::NotTried => { panic!("Not tried after trying"); }
        }
    })
}

pub fn cache_constant<'f,'a,T: 'a>(input: Value<'f,'a,T>) -> Value<'f,'a,Arc<T>> {
    do_cache_constant(input,|x| Arc::new(x))
}

pub fn cache_constant_clonable<'f:'a,'a,T: 'a+Clone>(input: Value<'f,'a,T>) -> Value<'f,'a,T> {
    do_cache_constant(input,|x| Arc::new(x)).dearc()
}

pub fn cache_constant_arc<'f,'a,T: 'a>(input: Value<'f,'a,Arc<T>>) -> Value<'f,'a,Arc<T>> {
    do_cache_constant(input,|x| x)
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::{puzzle3::{AnswerAllocator, derived, short_unknown_promise_clonable, Value}, lock};

    use super::{constant, cache_constant_clonable};

    #[test]
    fn constant_smoke() {
        let mut a = AnswerAllocator::new();
        let c = constant(45);
        assert_eq!(Some(45),c.constant());
        assert_eq!(45,c.call(&a.get()));
    }

    #[test]
    fn cache_constant_smoke() {
        let mut a = AnswerAllocator::new();
        let count = Arc::new(Mutex::new(0));
        let count2 = count.clone();
        let c = constant(12);
        let d = derived(c,|x| { *lock!(count2) += 1; x*x});
        let uc = cache_constant_clonable(d.clone());
        assert_eq!(144,uc.call(&mut a.get()));
        assert_eq!(Some(144),uc.constant());
        uc.call(&mut a.get());
        uc.call(&mut a.get());
        assert_eq!(1,*lock!(count));
        d.call(&mut a.get());
        assert_eq!(2,*lock!(count));
        /**/
        let(_,u) : (_,Value<i32>) = short_unknown_promise_clonable();
        let d = derived(u,|x| x*x);
        let uc = cache_constant_clonable(d.clone());
        assert_eq!(None,uc.constant());
    }
}
