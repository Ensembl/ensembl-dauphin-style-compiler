use std::{sync::{Arc, Mutex}};

use crate::lock;

use super::solver::{Solver};

pub fn constant_solver<'f,'a,T: Clone + 'a>(value: T) -> Solver<'f,'a,T> {
    Solver::new(move |_| Some(value.clone()))
}

enum ConstantResult<T> {
    NotTried,
    NotAvailable,
    Available(Arc<T>)
}

impl<T> ConstantResult<T> {
    fn try_constant<'f,'a,F,U>(&mut self, input: &Solver<'f,'a,U>, arcify: F) -> Option<Arc<T>>  where F: Fn(U) -> Arc<T> + 'f {
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

pub fn do_use_constant<'a,'f, T: 'a,U: 'a, F: 'f>(input: Solver<'f,'a,U>, arcify: F) -> Solver<'f,'a,Arc<T>>
        where F: Fn(U) -> Arc<T> + 'f {
    let constant = Arc::new(Mutex::new(ConstantResult::NotTried));
    let arcify = Arc::new(arcify);
    Solver::new(move |answer_index| {
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

pub fn use_constant<'f,'a,T: 'a>(input: Solver<'f,'a,T>) -> Solver<'f,'a,Arc<T>> {
    do_use_constant(input,|x| Arc::new(x))
}

pub fn use_constant_clonable<'f:'a,'a,T: 'a+Clone>(input: Solver<'f,'a,T>) -> Solver<'f,'a,T> {
    do_use_constant(input,|x| Arc::new(x)).dearc()
}

pub fn use_constant_arc<'f,'a,T: 'a>(input: Solver<'f,'a,Arc<T>>) -> Solver<'f,'a,Arc<T>> {
    do_use_constant(input,|x| x)
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use crate::{puzzle3::{combination::{combine_slice, derived}, constant::{constant_solver, use_constant}, answer::AnswerIndexAllocator}, lock};
    
    #[test]
    fn array_flattening() {
        for do_memoize in &[false,true] {
            let count = Arc::new(Mutex::new(0));
            /* consts will respond to None directly ... */
            let mut consts = vec![];
            for i in 0..10 {
                consts.push(constant_solver(i));
            }
            /* ... but deriveds will not */
            let mut deriveds = vec![];
            for c in &consts {
                let c = c.clone();
                let count2 = count.clone();
                deriveds.push(derived(c,move |v| {
                    *lock!(count2) += 1;
                    v*v
                }));
            }
            /* Let's build them into a single array and memoize that */
            let value = combine_slice(&deriveds, |v| v.to_vec());
            let value = if *do_memoize {
                use_constant(value)
            } else {
                derived(value,|value| Arc::new(value))
            };
            /* Make a single sum value, for easy testing */
            let total = derived(value,move |value| {
                value.iter().fold(0,|a,b| a+*b)
            });
            /* Evaluate twice (to try to trick into calling deriveds twice: shouldn't as we have memoized) */
            let mut aia = AnswerIndexAllocator::new();
            let mut ai1 = aia.get_answer_index();
            let mut ai2 = aia.get_answer_index();
            let v1 = total.call(&mut ai1);
            let v2 = total.call(&mut ai2);
            assert_eq!(285,v1); /* 1+4+...+64+81 */
            assert_eq!(285,v2);
            assert_eq!(if *do_memoize { 10 } else { 20 },*lock!(count));
        }
    }
}
