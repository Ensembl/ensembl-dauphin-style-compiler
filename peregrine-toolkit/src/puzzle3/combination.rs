use std::sync::Arc;

use super::{solver::Solver};

pub fn derived<'a, 'b: 'a, 'f: 'a+'b, 'g: 'f, 'h, T:'a, U:'b, F: 'f>(a: Solver<'g,'a,T>, f: F) -> Solver<'h,'b,U> where F: Fn(T) -> U {
    Solver::new(move |answer_index| {
        a.inner(answer_index).map(|a| f(a))
    })
}

pub fn combine2<'a, 'b, 'c:'a+'b, 'f:'a+'b+'c, 'g:'f, 'h:'f, 'j, T:'a, U:'b, V:'c, F:'f>(a: Solver<'g,'a,T>, b: Solver<'h,'b,U>, f: F) -> Solver<'j,'c,V> 
        where F: Fn(T,U) -> V {
    Solver::new(move |answer_index| {
        let (a,b) = (a.inner(answer_index),b.inner(answer_index));
        if a.is_none() || b.is_none() { return None; }
        Some(f(a.unwrap(),b.unwrap()))
    })
}

pub fn combine_slice<'a:'b, 'b, 'f:'a+'b, 'g:'f, T, U, F:'f>(inputs: &[Solver<'g,'a,T>], f: F) -> Solver<'f,'b,U> where F: Fn(&[T]) -> U {
    let inputs = Arc::new(inputs.iter().cloned().collect::<Vec<_>>());
    Solver::new(move |answer_index| {
        let mut values = vec![];
        for input in &*inputs {
            if let Some(value) = input.inner(answer_index) {
                values.push(value)
            } else {
                return None;
            }
        }
        Some(f(&values))
    })
}
