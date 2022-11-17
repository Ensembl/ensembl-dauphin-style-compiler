use std::{rc::Rc, cell::RefCell};

use super::value::{Value};

pub fn constant<'f:'a,'a,T: Clone + 'a>(value: T) -> Value<'f,'a,T> {
    Value::new(move |_| Some(value.clone()))
}

enum ConstantResult<T> {
    NotTried,
    NotAvailable,
    Available(Rc<T>)
}

impl<T> ConstantResult<T> {
    fn try_constant<'f:'a,'a,F,U>(&mut self, input: &Value<'f,'a,U>, rcify: F) -> Option<Rc<T>>  where F: Fn(U) -> Rc<T> {
        if let Some(value) = input.constant() {
            let value = rcify(value);
            *self = ConstantResult::Available(value.clone());
            Some(value)
        } else {
            *self = ConstantResult::NotAvailable;
            None
        }

    }
}

fn do_cache_constant<'a,'f:'a, T: 'a,U: 'a, F: 'f>(input: Value<'f,'a,U>, rcify: F) -> Value<'f,'a,Rc<T>>
        where F: Fn(U) -> Rc<T> {
    let constant = Rc::new(RefCell::new(ConstantResult::NotTried));
    let rcify = Rc::new(rcify);
    Value::new(move |answer_index| {
        let mut cache = constant.borrow_mut();
        if let ConstantResult::NotTried = &*cache {
            cache.try_constant(&input,&*rcify);
        }
        match &*cache {
            ConstantResult::NotAvailable => {
                input.inner(answer_index).map(|x| rcify(x))
            },
            ConstantResult::Available(v) => Some(v.clone()),
            ConstantResult::NotTried => { panic!("Not tried after trying"); }
        }
    })
}

pub fn cache_constant<'f:'a,'a,T: 'a>(input: Value<'f,'a,T>) -> Value<'f,'a,Rc<T>> {
    do_cache_constant(input,|x| Rc::new(x))
}

pub fn cache_constant_clonable<'f:'a,'a,T: 'a+Clone>(input: Value<'f,'a,T>) -> Value<'f,'a,T> {
    do_cache_constant(input,|x| Rc::new(x)).derc()
}

pub fn cache_constant_rc<'f:'a,'a,T: 'a>(input: Value<'f,'a,Rc<T>>) -> Value<'f,'a,Rc<T>> {
    do_cache_constant(input,|x| x)
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};
    use crate::{puzzle::{AnswerAllocator, derived, short_unknown_promise_clonable, Value}};
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
        let count = Rc::new(RefCell::new(0));
        let count2 = count.clone();
        let c = constant(12);
        let d = derived(c,|x| { *count2.borrow_mut() += 1; x*x});
        let uc = cache_constant_clonable(d.clone());
        assert_eq!(144,uc.call(&mut a.get()));
        assert_eq!(Some(144),uc.constant());
        uc.call(&mut a.get());
        uc.call(&mut a.get());
        assert_eq!(1,*count.borrow());
        d.call(&mut a.get());
        assert_eq!(2,*count.borrow());
        /**/
        let(_,u) : (_,Value<i32>) = short_unknown_promise_clonable();
        let d = derived(u,|x| x*x);
        let uc = cache_constant_clonable(d.clone());
        assert_eq!(None,uc.constant());
    }
}
