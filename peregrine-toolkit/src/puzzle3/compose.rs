use std::sync::Arc;

use super::{value::Value};

pub fn derived<'a: 'b+'g, 'b, 'f: 'a+'b, 'g: 'f, 'h: 'g, T:'a, U:'b, F: 'f>(a: Value<'g,'a,T>, f: F) -> Value<'h,'b,U> where F: Fn(T) -> U {
    Value::new(move |answer_index| {
        a.inner(answer_index).map(|a| f(a))
    })
}

pub fn compose<'a, 'b, 'c:'a+'b, 'f:'a+'b+'c, 'g:'f, 'h:'f, 'j, T:'a, U:'b, V:'c, F:'f>(a: Value<'g,'a,T>, b: Value<'h,'b,U>, f: F) -> Value<'j,'c,V> 
        where F: Fn(T,U) -> V {
            Value::new(move |answer_index| {
        let (a,b) = (a.inner(answer_index),b.inner(answer_index));
        if a.is_none() || b.is_none() { return None; }
        Some(f(a.unwrap(),b.unwrap()))
    })
}

pub fn compose_slice<'a:'b, 'b, 'f:'a+'b, 'g:'f, T, U, F:'f>(inputs: &[Value<'g,'a,T>], f: F) -> Value<'f,'b,U> where F: Fn(&[T]) -> U {
    let inputs = Arc::new(inputs.iter().cloned().collect::<Vec<_>>());
    Value::new(move |answer_index| {
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
