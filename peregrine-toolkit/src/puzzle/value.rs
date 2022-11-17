/* puzzle is a means of building a complex calculation at runtime and which can then be run with different
 * input values for the unknowns. Compared to using closures directly, puzzle localises argument lists to 
 * remove the need for plumbing, or knowing exactly what will vary for a given structure. For example, during
 * construction a function A might generate a value which is variable and function B might generate a constant
 * expression. These look the same to any "outer" function which might compose them or choose between them. Only
 * code associated with a leaf containing an unknown need be aware of the need to manage that unknown on each
 * call. Puzzle  has a mechanism for detecting and exploiting constants (ie fixed values) and not recomputing
 * them for each calculation. This makes it efficient in situations where most values can be pre-calculated and
 * only a few are unknown.
 * 
 * The core datastructure is Value. You can create these directly with the constructor if you like but they have
 * complex semantics and it's almost always safer and more convenient to use one of the utility functions. A
 * value has a type T which it represents, and two lifetimes, the first is the lifetime of the Value itself and
 * the second thelifetime of the contained type. The key externally visible method is Value.call(answer) which
 * returns a value T for the given answer.
 * 
 * An answer is a type which is used for each run of the computation. It is passed to the setter methods for
 * unknowns along with the value for that answer, and to Value.call() to find the result. See the unit tests
 * for usage examples.
 */

use std::rc::Rc;

use super::{answer::Answer, derived };

pub struct Value<'f,'a: 'f,T: 'a> {
    f: Rc<dyn (Fn(&Option<&Answer<'a>>) -> Option<T>) + 'f>
}

impl<'f,'a,T> Clone for Value<'f,'a,T> {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl<'f:'a, 'a,T: 'a> Value<'f,'a,T> {
    pub fn new<F>(f: F) -> Value<'f,'a,T> 
            where F:  (Fn(&Option<&Answer<'a>>) -> Option<T>) + 'f {
        Value { f: Rc::new(f) } 
    }

    pub fn inner(&self, index: &Option<&Answer<'a>>) -> Option<T> { (self.f)(index) }
    pub fn call(&self, index: &Answer<'a>) -> T { (self.f)(&Some(index)).unwrap() }
    pub fn constant(&self) -> Option<T> { (self.f)(&mut None) }
}

impl<'f:'a,'a,T:'a+Clone> Value<'f,'a,Rc<T>> {
    pub fn derc(self) -> Value<'f,'a,T> {
        derived(self,|x| x.as_ref().clone())
    }
}

impl<'f:'a,'a,T:'a> Value<'f,'a,Option<T>> {
    pub fn expect(self, msg: &str)  -> Value<'f,'a,T> {
        let msg = msg.to_string();
        derived(self,move |x| x.expect(&msg))
    }
}

pub type StaticValue<T> = Value<'static,'static,T>;
