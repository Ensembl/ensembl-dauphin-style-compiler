use crate::lock;
use std::sync::{ Arc, Mutex };
use commander::PromiseFuture;

// TODO move to commander
struct FuseState<V> {
    fused: Option<V>,
    promises: Vec<PromiseFuture<V>>
}

impl<V> FuseState<V> where V: Clone {
    fn new() -> FuseState<V> {
        FuseState {
            fused: None,
            promises: vec![]
        }
    }

    fn add(&mut self, promise: PromiseFuture<V>) {
        if let Some(value) = &self.fused {
            promise.satisfy(value.clone());
        } else {
            self.promises.push(promise);
        }
    }

    fn fuse(&mut self, value: V) {
        self.fused = Some(value.clone());
        for p in &self.promises {
            p.satisfy(value.clone());
        }
    }
}

#[derive(Clone)]
pub struct FusePromise<V>(Arc<Mutex<FuseState<V>>>);

impl<V> FusePromise<V> where V: Clone {
    pub fn new() -> FusePromise<V> {
        FusePromise(Arc::new(Mutex::new(FuseState::new())))
    }

    pub fn add(&self, promise: PromiseFuture<V>) {
        lock!(self.0).add(promise);
    }

    pub fn fuse(&self, value: V) {
        lock!(self.0).fuse(value);
    }
}
