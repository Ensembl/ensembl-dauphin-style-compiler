use std::sync::{Arc, Mutex};

use crate::lock;

use super::solver::Solver;

#[derive(Clone)]
pub struct SolverSetter<'f,'a,T: 'a>(Arc<Mutex<Option<Arc<Solver<'f,'a,T>>>>>);

pub fn delayed_solver<'f,'a,T: 'a>() -> (SolverSetter<'f,'a,T>,Solver<'f,'a,Option<T>>) {
    let value = Arc::new(Mutex::new(None));
    let value2 = value.clone();
    (SolverSetter(value),Solver::new(move |answer_index| {
        lock!(value2).as_ref().map(|x| x.inner(answer_index))
    }))
}

impl<'f,'a,T> SolverSetter<'f,'a,T> {
    pub fn set(&self, solver: Solver<'f,'a,T>) {
        *lock!(self.0) = Some(Arc::new(solver))
    }
}
