use std::sync::{ Arc, Mutex };
use crate::PromiseFuture;

trait DownstreamFuse<F> {
    fn fuse(&mut self, value: F);
}

struct Downstream<F,G> where G: Clone {
    state: FusePromise<G>,
    map: Option<Box<dyn FnOnce(F) -> G>>
}

impl<F,G> DownstreamFuse<F> for Downstream<F,G> where G: Clone {
    fn fuse(&mut self, value: F) {
        if let Some(map) = self.map.take() {
            self.state.fuse(map(value));
        }
    }
}

struct FuseState<V> {
    fused: Option<V>,
    promises: Vec<PromiseFuture<V>>,
    downstreams: Vec<Box<dyn DownstreamFuse<V> + 'static>>
}

impl<V> FuseState<V> where V: Clone {
    fn new() -> FuseState<V> {
        FuseState {
            fused: None,
            promises: vec![],
            downstreams: vec![]
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
        if self.fused.is_some() { return; }
        self.fused = Some(value.clone());
        for p in &self.promises {
            p.satisfy(value.clone());
        }
        for d in &mut self.downstreams {
            d.fuse(value.clone());
        }
    }
}

impl<V> FuseState<V> where V: Clone + 'static {
    fn add_downstream<F,G>(&mut self, fuse: &FusePromise<G>, map: F) where F: FnOnce(V) -> G + 'static, G: Clone + 'static {
        let mut downstream = Downstream {
            state: fuse.clone(),
            map: Some(Box::new(map))
        };
        if let Some(value) = &self.fused {
            downstream.fuse(value.clone());
        }
        self.downstreams.push(Box::new(downstream));
    }
}

#[derive(Clone)]
/// Just like a PromiseFuture except it can be waited on many times as V is clone.
pub struct FusePromise<V>(Arc<Mutex<FuseState<V>>>) where V: Clone;

impl<V> FusePromise<V> where V: Clone {
    /// Create a new FusePromise
    pub fn new() -> FusePromise<V> {
        FusePromise(Arc::new(Mutex::new(FuseState::new())))
    }

    /// Add a PromiseFuture to be satisfied when `fuse()` has been called.
    pub fn add(&self, promise: PromiseFuture<V>) {
        self.0.lock().unwrap().add(promise);
    }

    /// Satisfy all current and future added `PromiseFuture`s
    pub fn fuse(&self, value: V) {
        self.0.lock().unwrap().fuse(value);
    }
}

impl<V> FusePromise<V> where V: Clone + 'static {
    pub fn add_downstream<F,G>(&self, fuse: &FusePromise<G>, map: F) where F: FnOnce(V) -> G + 'static, G: Clone + 'static {
        self.0.lock().unwrap().add_downstream(fuse,map);
    }
}