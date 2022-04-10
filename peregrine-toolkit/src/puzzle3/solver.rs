use std::sync::Arc;

use super::{answer::AnswerIndex, derived};

pub struct Solver<'f,'a: 'f,T: 'a> {
    f: Arc<dyn (Fn(&Option<&AnswerIndex<'a>>) -> Option<T>) + 'f>
}

impl<'f,'a,T> Clone for Solver<'f,'a,T> {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl<'f,'a: 'f,T: 'a> Solver<'f,'a,T> {
    pub fn new<F>(f: F) -> Solver<'f,'a,T> 
            where F:  (Fn(&Option<&AnswerIndex<'a>>) -> Option<T>) + 'f {
        Solver { f: Arc::new(f) } 
    }

    pub fn inner(&self, index: &Option<&AnswerIndex<'a>>) -> Option<T> { (self.f)(index) }
    pub fn call(&self, index: &AnswerIndex<'a>) -> T { (self.f)(&Some(index)).unwrap() }
    pub fn constant(&self) -> Option<T> { (self.f)(&mut None) }
}

impl<'f,'a,T:'a+Clone> Solver<'f,'a,Arc<T>> {
    pub fn dearc<'g,'b>(self)  -> Solver<'g,'b,T> where 'f:'b, 'g:'a {
        derived(self,|x| x.as_ref().clone())
    }
}

impl<'f,'a,T:'a> Solver<'f,'a,Option<T>> {
    pub fn unwrap<'g,'b>(self)  -> Solver<'g,'b,T> where 'f:'b, 'g:'a {
        derived(self,|x| x.unwrap())
    }
}


pub type StaticSolver<T> = Solver<'static,'static,T>;

/*TODO:

rename
unit test
document

*/
