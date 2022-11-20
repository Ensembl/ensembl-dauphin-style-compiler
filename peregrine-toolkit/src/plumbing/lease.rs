use std::{sync::{Arc, Mutex}};
use crate::{lock};

// CANNOT BE CLONE
pub struct Lease<X> {
    dropper: Box<dyn FnMut(X) + 'static>,
    value: Option<X>
}

impl<X> Drop for Lease<X> {
    fn drop(&mut self) {
        (self.dropper)(self.value.take().unwrap());
    }
}

impl<X> Lease<X> {
    pub fn new<F>(dropper: F, value: X) -> Lease<X>
            where F: FnMut(X) + 'static {
        Lease { dropper: Box::new(dropper), value: Some(value) }
    }

    pub fn get(&self) -> &X { self.value.as_ref().unwrap() }
    pub fn get_mut(&mut self) -> &mut X { self.value.as_mut().unwrap() }
}

#[derive(Clone)]
pub struct LeaseManager<X,E> where X: 'static {
    ctor: Arc<Mutex<dyn FnMut() -> Result<X,E>>>,
    stable: Arc<Mutex<Vec<X>>>
}

impl<X,E> LeaseManager<X,E> {
    pub fn new<F>(ctor: F) -> LeaseManager<X,E> where F: FnMut() -> Result<X,E> + 'static {
        LeaseManager {
            ctor: Arc::new(Mutex::new(ctor)),
            stable: Arc::new(Mutex::new(vec![]))
        }
    }

    pub fn allocate(&self) -> Result<Lease<X>,E> {
        let x = if let Some(x) = lock!(self.stable).pop() {
            x
        } else {
            lock!(self.ctor)()?
        };
        let stable = self.stable.clone();
        Ok(Lease::new(move |v| lock!(stable).push(v),x))
    }
}
