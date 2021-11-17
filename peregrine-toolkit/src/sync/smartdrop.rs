use std::sync::{Arc, Mutex};
use crate::lock;

pub struct SmartDrop<T> {
    refs: Arc<Mutex<i32>>,
    value: Arc<Mutex<Option<T>>>,
    callback: Arc<Mutex<Option<Box<dyn FnOnce(T)>>>>
}

impl<'t,T> Drop for SmartDrop<T> {
    fn drop(&mut self) {
        let mut refs = lock!(self.refs);
        *refs -= 1;
        if *refs == 0 {
            drop(refs);
            let cb = lock!(self.callback).take().unwrap();
            cb(lock!(self.value).take().unwrap());
        }
    }
}

impl<T: Clone> Clone for SmartDrop<T> {
    fn clone(&self) -> Self {
        *lock!(self.refs) += 1;
        SmartDrop {
            refs: self.refs.clone(),
            value: self.value.clone(),
            callback: self.callback.clone()
        }
    }
}

impl<T> SmartDrop<T> {
    fn new<F>(value: T, cb: F) -> SmartDrop<T> where F: FnOnce(T) + 'static {
        SmartDrop {
            refs: Arc::new(Mutex::new(1)),
            value: Arc::new(Mutex::new(Some(value))),
            callback: Arc::new(Mutex::new(Some(Box::new(cb))))
        }
    }
}
