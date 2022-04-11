use std::sync::Arc;

use super::{answer::Answer, derived};

pub struct Value<'f,'a: 'f,T: 'a> {
    f: Arc<dyn (Fn(&Option<&Answer<'a>>) -> Option<T>) + 'f>
}

impl<'f,'a,T> Clone for Value<'f,'a,T> {
    fn clone(&self) -> Self {
        Self { f: self.f.clone() }
    }
}

impl<'f,'a: 'f,T: 'a> Value<'f,'a,T> {
    pub fn new<F>(f: F) -> Value<'f,'a,T> 
            where F:  (Fn(&Option<&Answer<'a>>) -> Option<T>) + 'f {
        Value { f: Arc::new(f) } 
    }

    pub fn inner(&self, index: &Option<&Answer<'a>>) -> Option<T> { (self.f)(index) }
    pub fn call(&self, index: &Answer<'a>) -> T { (self.f)(&Some(index)).unwrap() }
    pub fn constant(&self) -> Option<T> { (self.f)(&mut None) }
}

impl<'f,'a,T:'a+Clone> Value<'f,'a,Arc<T>> {
    pub fn dearc<'g,'b>(self)  -> Value<'g,'b,T> where 'f:'b, 'g:'a {
        derived(self,|x| x.as_ref().clone())
    }
}

impl<'f,'a,T:'a> Value<'f,'a,Option<T>> {
    pub fn unwrap<'g,'b>(self)  -> Value<'g,'b,T> where 'f:'b, 'g:'a {
        derived(self,|x| x.unwrap())
    }
}


pub type StaticValue<T> = Value<'static,'static,T>;
