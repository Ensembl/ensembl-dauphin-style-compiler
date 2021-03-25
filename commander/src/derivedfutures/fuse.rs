use std::sync::{ Arc, Mutex };
use crate::PromiseFuture;

/// Just like a PromiseFuture except it can be waited on many times as V is clone.
struct FuseState<V> {
    fused: Option<V>,
    promises: Vec<PromiseFuture<V>>
}

impl<V> FuseState<V> where V: Clone {
    /// Create a new FusePromise
    fn new() -> FuseState<V> {
        FuseState {
            fused: None,
            promises: vec![]
        }
    }

    /// Add a PromiseFuture to be satisfied when `fuse()` has been called.
    fn add(&mut self, promise: PromiseFuture<V>) {
        if let Some(value) = &self.fused {
            promise.satisfy(value.clone());
        } else {
            self.promises.push(promise);
        }
    }

    /// Satisfy all current and future added `PromiseFuture`s
    fn fuse(&mut self, value: V) {
        if self.fused.is_some() { return; }
        self.fused = Some(value.clone());
        for p in &self.promises {
            p.satisfy(value.clone());
        }
    }
}

#[derive(Clone)]
pub struct FusePromise<V>(Arc<Mutex<FuseState<V>>>) where V: Clone;

impl<V> FusePromise<V> where V: Clone {
    pub fn new() -> FusePromise<V> {
        FusePromise(Arc::new(Mutex::new(FuseState::new())))
    }

    pub fn add(&self, promise: PromiseFuture<V>) {
        self.0.lock().unwrap().add(promise);
    }

    pub fn fuse(&self, value: V) {
        self.0.lock().unwrap().fuse(value);
    }
}
