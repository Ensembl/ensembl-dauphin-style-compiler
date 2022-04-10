use std::sync::Arc;

use super::{answer::AnswerIndex, solver::Solver, SolverSetter, delayed_solver, derived};

struct ClonableCommuter<'f,'a,T: Clone> {
    initial: T,
    compose: Box<dyn Fn(&T,&T) -> T + 'f>,
    rest: Vec<Solver<'f,'a,T>>
}

impl<'f,'a,T: Clone> ClonableCommuter<'f,'a,T> {
    fn new<F: 'a>(initial: T, compose: F) -> ClonableCommuter<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
        ClonableCommuter { initial, compose: Box::new(compose), rest: vec![] }
    }

    fn add(&mut self, solver: Solver<'f,'a,T>) {
        if let Some(constant) = solver.constant() {
            self.initial = (self.compose)(&self.initial,&constant);
        } else {
            self.rest.push(solver);
        }
    }

    fn inner(&self, answer_index: &Option<&AnswerIndex<'a>>) -> Option<T> {
        let mut out = self.initial.clone();
        for var in &self.rest {
            let value = var.inner(answer_index);
            if let Some(value) = value {
                out = (self.compose)(&out,&value);
            } else {
                return None;
            }
        }
        Some(out)
    }
}

pub struct Commuter<'f,'a,T>(ClonableCommuter<'f,'a,Arc<T>>);

impl<'f:'a,'a,T> Commuter<'f,'a,T> {
    fn new<F>(initial: T, compose: F) -> Commuter<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
        Commuter(ClonableCommuter::new(Arc::new(initial),move |a,b|{
            Arc::new(compose(&*a,&*b))
        }))
    }

    fn add<'g,'b>(&mut self, solver: Solver<'f,'a,T>) where 'g:'b, 'f:'a {
        self.0.add(derived(solver,|x| Arc::new(x)))
    }

    fn inner(&self, answer_index: &Option<&AnswerIndex<'a>>) -> Option<Arc<T>> {
        self.0.inner(answer_index)
    }

}

pub struct ArcCommuter<'f,'a,T>(ClonableCommuter<'f,'a,Arc<T>>);

impl<'f: 'a,'a,T> ArcCommuter<'f,'a,T> {
    fn new(initial: Arc<T>, compose: Arc<dyn Fn(&T,&T) -> T + 'f>) -> ArcCommuter<'f,'a,T> {
        ArcCommuter(ClonableCommuter::new(initial,move |a,b| {
            Arc::new(compose(&*a,&*b))
        }))
    }

    fn add(&mut self, solver: Solver<'f,'a,Arc<T>>) {
        self.0.add(solver);
    }

    fn inner(&self, answer_index: &Option<&AnswerIndex<'a>>) -> Option<Arc<T>> {
        self.0.inner(answer_index)
    }
}


pub fn commute<'f: 'a,'a,T: 'a,F: 'f>(inputs: &[Solver<'f,'a,T>], initial: T, compose: F) -> Solver<'f,'a,Arc<T>> where F: Fn(&T,&T) -> T + 'f {
    let mut commuter = Commuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Solver::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub fn commute_clonable<'f: 'a,'a,T: 'a+Clone,F: 'f>(inputs: &[Solver<'f,'a,T>], initial: T, compose: F) -> Solver<'f,'a,T> where F: Fn(&T,&T) -> T + 'f {
    let mut commuter = ClonableCommuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Solver::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub fn commute_arc<'f: 'a,'a,T: 'a>(inputs: &[Solver<'f,'a,Arc<T>>], initial: Arc<T>, compose: Arc<dyn Fn(&T,&T) -> T + 'f>) -> Solver<'f,'a,Arc<T>> {
    let mut commuter = ArcCommuter::new(initial,compose);
    for input in inputs {
        commuter.add(input.clone());
    }
    Solver::new(move |answer_index| {
        commuter.inner(answer_index)
    })
}

pub struct DelayedCommuteBuilder<'a,T: 'a> {
    f: Arc<dyn Fn(&T,&T) -> T + 'a>,
    solver: Solver<'a,'a,Arc<T>>,
    setter: SolverSetter<'a,'a,Arc<T>>,
    values: Vec<Solver<'a,'a,Arc<T>>>
}

impl<'a,T: 'a> DelayedCommuteBuilder<'a,T> {
    pub fn new<F>(f: F) -> DelayedCommuteBuilder<'a,T> where F: Fn(&T,&T) -> T + 'a {
        let (setter,solver) = delayed_solver();
        let solver = solver.unwrap();
        DelayedCommuteBuilder { setter, solver, f: Arc::new(f), values: vec![] }
    }

    pub fn solver(&self) -> &Solver<'a,'a,Arc<T>> { &self.solver }

    pub fn add(&mut self, value: &Solver<'a,'a,T>) { 
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

    use crate::{lock, puzzle3::{constant::constant_solver, unknown::short_unknown, combination::derived, answer::AnswerIndexAllocator}};

    use super::commute;

    #[test]
    fn commute_smoke() {
        let count = Arc::new(Mutex::new(0));
        let count2 = count.clone();
        /* evens are const, odds are variable */
        let mut inputs = vec![];
        let mut sets = vec![];
        for i in 0..10 {
            if i%2 == 0 {
                let constant = constant_solver(i);
                /* derive so that we can capture counts */
                let value = derived(constant,|v| {
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
        /* Put into a commute: should pick up the constants */
        let total = commute(&inputs,0,|a,b| {
           *a+*b
        });
        /* 1: set the odds to their usual values, 2: set odds to 0 */
        let mut aia = AnswerIndexAllocator::new();
        let mut ai1 = aia.get_answer_index();
        let mut ai2 = aia.get_answer_index();
        for (i,s) in sets.iter_mut().enumerate() {
            s.set(&mut ai1, (i*2+1)*(i*2+1));
            s.set(&mut ai2, 0);
        }
        println!("ready!");
        let v1 = total.call(&mut ai1);
        let v2 = total.call(&mut ai2);
        assert_eq!(285,*v1); /* 1+4+...+64+81 */
        assert_eq!(120,*v2); /* 4+16+...+16+64 */
        assert_eq!(5,*lock!(count));
    }
}
