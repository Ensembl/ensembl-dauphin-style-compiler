use std::sync::{Arc, Mutex};

use crate::lock;

use super::solver::Solver;

#[derive(Clone)]
pub struct SolverSetter<'f,'a: 'f,T: 'a>(Arc<Mutex<Option<Arc<Solver<'f,'a,T>>>>>);

pub fn delayed_solver<'f,'a: 'f,T: 'a>() -> (SolverSetter<'f,'a,T>,Solver<'f,'a,Option<T>>) {
    let value = Arc::new(Mutex::new(None));
    let value2 = value.clone();
    (SolverSetter(value),Solver::new(move |answer_index| {
        if let Some(inner) = &*lock!(value2) {
            /* value has been set, return it */
            Some(inner.inner(answer_index))
        } else {
            if answer_index.is_some() {
                /* value has not been set and call is real, return None */
                Some(None)
            } else {
                /* value has not been set, but is constant */
                None
            }
        }
    }))
}

impl<'f,'a,T> SolverSetter<'f,'a,T> {
    pub fn set(&self, solver: Solver<'f,'a,T>) {
        *lock!(self.0) = Some(Arc::new(solver))
    }
}
