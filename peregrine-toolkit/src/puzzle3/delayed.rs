use std::sync::{Arc, Mutex};

use crate::lock;

use super::value::Value;

#[derive(Clone)]
pub struct SolverSetter<'f,'a: 'f,T: 'a>(Arc<Mutex<Option<Arc<Value<'f,'a,T>>>>>);

pub fn delayed<'f,'a: 'f,T: 'a>() -> (SolverSetter<'f,'a,T>,Value<'f,'a,Option<T>>) {
    let value = Arc::new(Mutex::new(None));
    let value2 = value.clone();
    (SolverSetter(value),Value::new(move |answer_index| {
        if let Some(inner) = &*lock!(value2) {
            /* value has been set, return it */
            if answer_index.is_some() {
                Some(inner.inner(answer_index))
            } else {
                inner.inner(answer_index).map(|x| Some(x))
            }
        } else {
            if answer_index.is_some() {
                /* value has not been set and call is real, return None */
                Some(None)
            } else {
                /* value has not been set, but is constant request */
                None
            }
        }
    }))
}

pub fn promise_delayed<'f,'a,'g,'b,T>() -> (SolverSetter<'f,'a,T>,Value<'g,'b,T>) where 'f:'b, 'g:'a {
    let (setter,solver) = delayed();
    (setter,solver.unwrap())
}

impl<'f,'a,T> SolverSetter<'f,'a,T> {
    pub fn set(&self, solver: Value<'f,'a,T>) {
        *lock!(self.0) = Some(Arc::new(solver))
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::puzzle3::{derived, constant, AnswerAllocator, promise_delayed, short_unknown};

    use super::delayed;


    #[test]
    fn delayed_smoke() {
        let mut a = AnswerAllocator::new();
        let (ds,d) = promise_delayed();
        let v = derived(d,|x| x*x);
        let x = derived(constant(11),|x| x-1);
        ds.set(x);
        assert_eq!(100,v.call(&mut a.get()));
        assert_eq!(Some(100),v.constant());
    }

    #[test]
    fn delayed_unset() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let (ds,d) = delayed();
        assert_eq!(None,d.constant());
        assert_eq!(None,d.call(&mut a.get()));
        let (mut us,u) = short_unknown();
        ds.set(u);
        assert_eq!(None,d.constant());
        assert_eq!(Some(None),d.call(&mut a1));
        us.set(&mut a1,16);
        assert_eq!(None,d.constant());
        assert_eq!(Some(Some(Arc::new(16))),d.call(&mut a1));
    }
}
