use std::rc::Rc;

use super::{value::Value};
/* Used by Value's callback:
 *   Input type must last longer than input Value (uses it): a > g
 *   Output type must last longer than output type: b > h
 * Used by our derivation callback:
 *   Input type must last longer than callback (uses it): a > f
 *   Output type must last longer than callback (creates it): b > f
 *   Uses input to generate output b > a
 * Derivation callback stored in output Value: f > h
 * Derivation callback uses input Value: f > g
 *
 * b -> a -> f -> {g,h}
 * 
 * 'a:'b, 'b, 'f:'a, 'g:'f, 'h:'f
 */

pub fn derived<'a:'b, 'b, 'f:'a, 'g:'a, 'h:'f, T:'a, U:'b, F: 'f>(a: Value<'g,'a,T>, f: F) -> Value<'h,'b,U> where F: Fn(T) -> U {
    Value::new(move |answer| {
        a.inner(answer).map(|a| f(a))
    })
}

#[cfg(debug_assertions)]
pub fn derived_debug<'a:'b, 'b, 'f:'a, 'g:'a, 'h:'f, T:'a, U:'b, F: 'f>(a: Value<'g,'a,T>, f: F, name: &str) -> Value<'h,'b,U> where F: Fn(T,Option<u64>) -> U {
    use crate::log;

    let name = name.to_string();
    Value::new(move |answer| {
        a.inner(answer).map(|a| {
            log!("enter: {}",name);
            let out = f(a,answer.map(|x| x.serial()));
            log!("leave: {}",name);
            out
        })
    })
}

/* Used by Value's callback:
 *   Input1 type must last longer than input1 Value: a > g
 *   Input2 type must last longer than input2 Value: b > h
 *   Output type must last longer than output Value: c > j
 * Used by our derivation callback:
 *   Input1 type must last longer than callback (used): a > f
 *   Input2 type must last longer than callback (used): b > f
 *   Output type must last longer than callback (created): c > f
 * Data can be transferred from input to output:
 *   Uses input1 to generate output: c > a
 *   Uses input2 to generate output: c > b
 * Derivation callback:
 *    stored in output Value: f > j
 *    uses input1 Value: f > g
 *    uses input2 Value: f > h
 *
 * so:
 * c -> {a,b} -> f -> {g,h,j}
 *
 * therefore:
 * 'a:'c, 'b:'c, 'c, 'f:'a+'b, 'g:'f, 'h:'f, 'j:'f
 */
pub fn compose<'a:'c, 'b:'c, 'c, 
               'f:'a+'b, 'g:'f, 'h:'f, 'j:'f,
                T:'a, U:'b, V:'c, F:'f>(a: Value<'g,'a,T>, b: Value<'h,'b,U>, f: F) -> Value<'j,'c,V> 
            where F: Fn(T,U) -> V {
    Value::new(move |answer_index| {
        let (a,b) = (a.inner(answer_index),b.inner(answer_index));
        if a.is_none() || b.is_none() { return None; }
        Some(f(a.unwrap(),b.unwrap()))
    })
}

/* lifetime comment same as for derived */
pub fn compose_slice<'a:'b, 'b, 'f:'a, 'g:'f, 'h:'f, T, U, F:'f>(inputs: &[Value<'g,'a,T>], f: F) -> Value<'h,'b,U> where F: Fn(&[T]) -> U {
    let inputs = Rc::new(inputs.iter().cloned().collect::<Vec<_>>());
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

/* lifetime comment same as for derived */
pub fn compose_slice_vec<'a:'b, 'b, 'f:'a, 'g:'f, 'h:'f, T>(inputs: &[Value<'g,'a,T>]) -> Value<'h,'b,Vec<T>> {
    let inputs = Rc::new(inputs.iter().cloned().collect::<Vec<_>>());
    Value::new(move |answer_index| {
        let mut values = vec![];
        for input in &*inputs {
            if let Some(value) = input.inner(answer_index) {
                values.push(value)
            } else {
                return None;
            }
        }
        Some(values)
    })
}

#[cfg(test)]
mod test {
    use compose::compose_slice;

    use crate::puzzle::{AnswerAllocator, short_unknown_promise_clonable, compose, constant, Value };

    use super::derived;

    #[test]
    fn derived_smoke() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let mut a2 = a.get();
        let (us,u) = short_unknown_promise_clonable();
        let d = derived(u,|v| v*v);
        us.set(&mut a1,6);
        us.set(&mut a2,7);
        assert_eq!(36,d.call(&a1));
        assert_eq!(49,d.call(&a2));
    }

    #[test]
    fn compose_smoke() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let mut a2 = a.get();
        let (us,u) = short_unknown_promise_clonable();
        let (vs,v) = short_unknown_promise_clonable();
        let d = compose(u,v,|u,v| u*v);
        us.set(&mut a1,7);
        us.set(&mut a2,8);
        vs.set(&mut a1,5);
        vs.set(&mut a2,6);
        assert_eq!(35,d.call(&a1));
        assert_eq!(48,d.call(&a2));
    }

    #[test]
    fn compose_slice_smoke() {
        let mut a = AnswerAllocator::new();
        let mut a1 = a.get();
        let mut a2 = a.get();
        let (us,u) = short_unknown_promise_clonable();
        let (vs,v) = short_unknown_promise_clonable();
        let d = compose_slice(&[u,v],|x| x[0]*x[1]);
        us.set(&mut a1,7);
        us.set(&mut a2,8);
        vs.set(&mut a1,5);
        vs.set(&mut a2,6);
        assert_eq!(35,d.call(&a1));
        assert_eq!(48,d.call(&a2));
    }

    #[test]
    fn derive_constant() {
        let c = constant(42);
        let (_,u) : (_,Value<i32>) = short_unknown_promise_clonable();
        let dc = derived(c,|x| x+2);
        let du = derived(u,|x| x+3);
        assert_eq!(Some(44),dc.constant());
        assert_eq!(None,du.constant());
    }

    #[test]
    fn compose_constant() {
        let (_,u) : (_,Value<i32>) = short_unknown_promise_clonable();
        let (_,v) : (_,Value<i32>) = short_unknown_promise_clonable();
        /* actually constant */
        let d1 = compose(constant(17),constant(31),|a,b| b-a);
        assert_eq!(Some(14),d1.constant());
        /* a or b not constant */
        let d2 = compose(constant(17),u.clone(),|a,b| b-a);
        assert_eq!(None,d2.constant());
        let d3 = compose(u.clone(),constant(31),|a,b| b-a);
        assert_eq!(None,d3.constant());
        /* neither constant */
        let d4 = compose(u,v,|a,b| b-a);
        assert_eq!(None,d4.constant());
    }

    #[test]
    fn compose_slice_constant() {
        let (_,u) : (_,Value<i32>) = short_unknown_promise_clonable();
        let (_,v) : (_,Value<i32>) = short_unknown_promise_clonable();
        /* actually constant */
        let d1 = compose_slice(&[constant(17),constant(31)],|x| x[1]-x[0]);
        assert_eq!(Some(14),d1.constant());
        /* a or b not constant */
        let d2 = compose_slice(&[constant(17),u.clone()],|x| x[1]-x[0]);
        assert_eq!(None,d2.constant());
        let d3 = compose_slice(&[u.clone(),constant(31)],|x| x[1]-x[0]);
        assert_eq!(None,d3.constant());
        /* neither constant */
        let d4 = compose_slice(&[u,v],|x| x[1]-x[0]);
        assert_eq!(None,d4.constant());
    }
}
