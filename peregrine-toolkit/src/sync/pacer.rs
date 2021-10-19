use std::sync::{Arc, Mutex};

use crate::lock;

struct PacerData<T> {
    config: Vec<T>,
    failed: Vec<bool>,
    offset: usize,
    count: usize
}

impl<T> PacerData<T> where T: Clone {
    fn new(config: &[T]) -> PacerData<T> {
        let config = config.to_vec();
        let failed = vec![false;config.len()];
        PacerData {
            config, failed,
            count: 0,
            offset: 0
        }
    }

    fn report(&mut self, failed: bool) {
        let delta_plus_one = match (self.failed[self.offset],failed) {
            (false,false) => 1,
            (false,true) => 2,
            (true,false) => 0,
            (true,true) => 1
        };
        self.failed[self.offset] = failed;
        self.count += delta_plus_one-1;
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
