use std::sync::{Arc, Mutex};

use peregrine_toolkit::lock;

struct PacerData<T> {
    config: Vec<T>,
    failed: Vec<bool>,
    offset: usize,
    count: usize
}

impl<T> PacerData<T> where T: Clone {
    fn new(config: &[T]) -> PacerData<T> {
        let config = config.to_vec();
        let failed = vec![false;config.len()-1];
        PacerData {
            config, failed,
            count: 0,
            offset: 0
        }
    }

    fn report(&mut self, failed: bool) {
        match (self.failed[self.offset],failed) {
            (false,true) => { self.count += 1; }
            (true,false) => { self.count -= 1; },
            _ => {}
        };
        self.failed[self.offset] = failed;
        self.offset = (self.offset+1) % self.failed.len();
    }

    fn get(&self) -> &T { &self.config[self.count] }
}

#[derive(Clone)]
pub struct Pacer<T>(Arc<Mutex<PacerData<T>>>);

impl<T: Clone> Pacer<T> {
    pub fn new(config: &[T]) -> Pacer<T> { Pacer(Arc::new(Mutex::new(PacerData::new(config)))) }
    pub fn report(&self, success: bool) { lock!(self.0).report(!success); }
    pub fn get(&self) -> T { lock!(self.0).get().clone() }
}
