use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Distributor<T>(Arc<Mutex<Vec<Box<dyn FnMut(&T) + 'static>>>>);

impl<T> Distributor<T> {
    pub fn new() -> Distributor<T> {
        Distributor(Arc::new(Mutex::new(vec![])))
    }

    pub fn add<F>(&mut self, cb: F) where F: FnMut(&T) + 'static {
        self.0.lock().unwrap().push(Box::new(cb));
    }

    pub fn send(&self, value: T) {
        let mut streams = self.0.lock().unwrap();
        for stream in streams.iter_mut() {
            stream(&value);
        }
    }
}
