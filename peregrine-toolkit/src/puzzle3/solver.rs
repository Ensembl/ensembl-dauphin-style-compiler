use std::sync::Arc;

use super::answer::AnswerIndex;

pub struct Solver<'f,'a: 'f,T: 'a> {
    f: Arc<dyn (Fn(&mut Option<&mut AnswerIndex<'a>>) -> Option<T>) + 'f>
}

impl<'f,'a,T> Clone for Solver<'f,'a,T> {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl<'f,'a,T> Solver<'f,'a,T> {
    pub fn new<F>(f: F) -> Solver<'f,'a,T> 
            where F:  (Fn(&mut Option<&mut AnswerIndex<'a>>) -> Option<T>) + 'f {
        Solver { f: Arc::new(f) } 
    }

    pub fn inner(&self, index: &mut Option<&mut AnswerIndex<'a>>) -> Option<T> { (self.f)(index) }
    pub fn call(&self, index: &mut AnswerIndex<'a>) -> T { (self.f)(&mut Some(index)).unwrap() }
    pub fn constant(&self) -> Option<T> { (self.f)(&mut None) }
}

