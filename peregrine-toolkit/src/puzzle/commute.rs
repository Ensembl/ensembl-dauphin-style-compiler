use std::sync::Arc;

use super::{answer::Answer, value::Value, DelayedSetter, delayed, derived };

struct ClonableBuildCommuter<'f:'a,'a,T> {
    initial: T,
    compose: Box<dyn Fn(&mut T,&T) + 'f>,
    cloner: Box<dyn Fn(&T) -> T + 'f>,
    rest: Vec<Value<'f,'a,T>>
}

impl<'f:'a,'a,T> ClonableBuildCommuter<'f,'a,T> {
    fn new<F: 'a, G>(initial: T, compose: F, cloner: G) -> ClonableBuildCommuter<'f,'a,T>
            where F: Fn(&mut T,&T) + 'f, G: Fn(&T) -> T + 'f {
        ClonableBuildCommuter { 
            initial, 
            compose: Box::new(compose), 
            cloner: Box::new(cloner),
            rest: vec![]
        }
    }

    fn add(&mut self, solver: Value<'f,'a,T>) {
        if let Some(constant) = solver.constant() {
            (self.compose)(&mut self.initial,&constant);
        } else {
            self.rest.push(solver);
        }
    }

    fn inner(&self, answer_index: &Option<&Answer<'a>>) -> Option<T> {
        let mut out = (self.cloner)(&self.initial);
        for var in &self.rest {
            let value = var.inner(answer_index);
            if let Some(value) = value {
                (self.compose)(&mut out,&value);
            } else {
                return None;
            }
        }
        Some(out)
    }
}

struct ClonableCommuter<'f:'a,'a,T: Clone>(ClonableBuildCommuter<'f,'a,T>);

impl<'f:'a,'a,T: Clone> ClonableCommuter<'f,'a,T> {
    fn new<F: 'a>(initial: T, compose: F) -> ClonableCommuter<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
        ClonableCommuter(ClonableBuildCommuter::new(initial,move |acc,extra| {
            *acc = compose(&*acc,extra);
        },|x| x.clone()))
    }

    fn add(&mut self, solver: Value<'f,'a,T>) {
        self.0.add(solver);
    }

    fn inner(&self, answer_index: &Option<&Answer<'a>>) -> Option<T> {
        self.0.inner(answer_index)
    }
}

struct Commuter<'f,'a,T>(ClonableCommuter<'f,'a,Arc<T>>);

impl<'f:'a,'a,T> Commuter<'f,'a,T> {
    fn new<F>(initial: T, compose: F) -> Commuter<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
        Commuter(ClonableCommuter::new(Arc::new(initial),move |a,b|{
            Arc::new(compose(&*a,&*b))
        }))
    }

    fn add<'g,'b>(&mut self, solver: Value<'f,'a,T>) where 'g:'b, 'f:'a {
        self.0.add(derived(solver,|x| Arc::new(x)))
    }

    fn inner(&self, answer_index: &Option<&Answer<'a>>) -> Option<Arc<T>> {
        self.0.inner(answer_index)
    }
}

struct ArcCommuter<'f,'a,T>(ClonableCommuter<'f,'a,Arc<T>>);

impl<'f: 'a,'a,T> ArcCommuter<'f,'a,T> {
    fn new(initial: Arc<T>, compose: Arc<dyn Fn(&T,&T) -> T + 'f>) -> ArcCommuter<'f,'a,T> {
        ArcCommuter(ClonableCommuter::new(initial,move |a,b| {
            Arc::new(compose(&*a,&*b))
        }))
    }

    fn add(&mut self, solver: Value<'f,'a,Arc<T>>) {
        self.0.add(solver);
    }

    fn inner(&self, answer_index: &Option<&Answer<'a>>) -> Option<Arc<T>> {
        self.0.inner(answer_index)
    }
}

pub fn commute<'f: 'a,'a,T: 'a,F: 'f>(inputs: &[Value<'f,'a,T>], initial: T, compose: F) -> Value<'f,'a,Arc<T>> where F: Fn(&T,&T) -> T + 'f {
    let mut commuter = Commuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Value::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub fn build_commute<'f:'a, 'a, T:'a+Clone, F:'f, G: 'f>(inputs: &[Value<'f,'a,T>], initial: T, compose: F, create: G) -> Value<'f,'a,T>
        where F: Fn(&mut T,&T) + 'f, G: Fn(&T) -> T {
    let mut commuter = ClonableBuildCommuter::new(initial,compose,create);
    for input in inputs {
        commuter.add(input.clone());
    }
    Value::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub fn commute_clonable<'f: 'a,'a,T: 'a+Clone,F: 'f>(inputs: &[Value<'f,'a,T>], initial: T, compose: F) -> Value<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
    let mut commuter = ClonableCommuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Value::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub fn commute_arc<'f: 'a,'a,T: 'a>(inputs: &[Value<'f,'a,Arc<T>>], initial: Arc<T>, compose: Arc<dyn Fn(&T,&T) -> T + 'f>) -> Value<'f,'a,Arc<T>> {
    let mut commuter = ArcCommuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Value::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub struct DelayedCommuteBuilder<'a,T: 'a> {
    f: Arc<dyn Fn(&T,&T) -> T + 'a>,
    solver: Value<'a,'a,Arc<T>>,
    setter: DelayedSetter<'a,'a,Arc<T>>,
    values: Vec<Value<'a,'a,Arc<T>>>
}

impl<'a,T: 'a> DelayedCommuteBuilder<'a,T> {
    pub fn new<F>(f: F) -> DelayedCommuteBuilder<'a,T> where F: Fn(&T,&T) -> T + 'a {
        let (setter,solver) = delayed();
        let solver = solver.unwrap();
        DelayedCommuteBuilder { setter, solver, f: Arc::new(f), values: vec![] }
    }

    pub fn solver(&self) -> &Value<'a,'a,Arc<T>> { &self.solver }

    pub fn add(&mut self, value: &Value<'a,'a,T>) { 
        let value = derived(value.clone(),|x| Arc::new(x));
        self.values.push(value);
    }

    pub fn build(&mut self, initial: T) {
        let value = commute_arc(&self.values,Arc::new(initial),self.f.clone());
        self.setter.set(value);
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};

    use unknown::{short_unknown_promise_clonable, UnknownSetter};

    use crate::{lock, puzzle::{constant::constant, unknown::{short_unknown, self}, compose::derived, answer::AnswerAllocator, Value, StaticValue, build_commute, derived_debug}};

    use super::{commute, DelayedCommuteBuilder};

    fn commute_smoke_setup(count: &Arc<Mutex<usize>>) -> (Vec<StaticValue<usize>>,Vec<UnknownSetter<'static,usize>>) {
        let count2 = count.clone();
        /* evens are const, odds are variable */
        let mut inputs = vec![];
        let mut sets = vec![];
        for i in 0..10 {
            if i%2 == 0 {
                let constant = constant(i);
                /* derive so that we can capture counts */
                let count2 = count2.clone();
                let value = derived(constant,move |v| {
                    *lock!(count2) += 1;
                    v*v
                });
                inputs.push(value);
            } else {
                let (setter,solver) = short_unknown();
                let value = derived(solver,|v: Option<Arc<usize>>| {
                    v.map(|x| (*x).clone()).unwrap().clone()
                });
                inputs.push(value);
                sets.push(setter);
            }
        }
        (inputs,sets)
    }

    fn commute_smoke_check(total: Value<'static,'static,usize>,sets: &mut Vec<UnknownSetter<'static,usize>>) {
        let mut aia = AnswerAllocator::new();
        let mut ai1 = aia.get();
        let mut ai2 = aia.get();
        for (i,s) in sets.iter_mut().enumerate() {
            s.set(&mut ai1, (i*2+1)*(i*2+1));
            s.set(&mut ai2, 0);
        }
        let v1 = total.call(&mut ai1);
        let v2 = total.call(&mut ai2);
        assert_eq!(285,v1); /* 1+4+...+64+81 */
        assert_eq!(120,v2); /* 4+16+...+16+64 */
    }

    #[test]
    fn commute_smoke() {
        let count = Arc::new(Mutex::new(0));
        let (inputs,mut sets) = commute_smoke_setup(&count);
        /* Put into a commute: should pick up the constants */
        let total = commute(&inputs,0,|a,b| {
           *a+*b
        }).dearc();
        commute_smoke_check(total,&mut sets);
        assert_eq!(5,*lock!(count));
    }

    #[test]
    fn build_commute_smoke() {
        let count = Arc::new(Mutex::new(0));
        let (inputs,mut sets) = commute_smoke_setup(&count);
        /* Put into a commute: should pick up the constants */
        let total = build_commute(&inputs,0,|a,b| {
           *a += *b
        },|x| *x);
        commute_smoke_check(total,&mut sets);
        assert_eq!(5,*lock!(count));
    }

    #[test]
    fn builder_smoke() {
        let mut a = AnswerAllocator::new();
        let mut b1 = DelayedCommuteBuilder::new(|a,b| *a+*b);
        let s1 = b1.solver().clone().dearc();
        b1.build(42);
        let a1 = a.get();
        assert_eq!(42,s1.call(&a1));
        /**/
        let mut b2 = DelayedCommuteBuilder::new(|a,b| *a+*b);
        let s2 = b2.solver().clone().dearc();
        b2.add(&constant(41));
        b2.build(42);
        let a2 = a.get();
        assert_eq!(83,s2.call(&a2));
        /**/
        let mut b3 = DelayedCommuteBuilder::new(|a,b| *a+*b);
        let s3 = b3.solver().clone().dearc();
        b3.add(&constant(41));
        let (mut u1s,u1) = short_unknown_promise_clonable();
        b3.add(&u1);
        b3.build(42);
        let mut a3a = a.get();
        let mut a3b = a.get();
        u1s.set(&mut a3a,12);
        u1s.set(&mut a3b,6);
        assert_eq!(95,s3.call(&a3a));
        assert_eq!(89,s3.call(&a3b));
    }
}
