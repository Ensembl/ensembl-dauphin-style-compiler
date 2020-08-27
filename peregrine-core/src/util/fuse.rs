use crate::lock;
use std::sync::{ Arc, Mutex };
use commander::PromiseFuture;

struct FuseState {
    fused: bool,
    promises: Vec<PromiseFuture<()>>
}

impl FuseState {
    fn new() -> FuseState {
        FuseState {
            fused: false,
            promises: vec![]
        }
    }

    fn add(&mut self, promise: PromiseFuture<()>) {
        if self.fused {
            promise.satisfy(());
        } else {
            self.promises.push(promise);
        }
    }

    fn fuse(&mut self) {
        self.fused = true;
        for p in &self.promises {
            p.satisfy(());
        }
    }
}

#[derive(Clone)]
pub struct FusePromise(Arc<Mutex<FuseState>>);

impl FusePromise {
    pub fn new() -> FusePromise {
        FusePromise(Arc::new(Mutex::new(FuseState::new())))
    }

    pub fn add(&mut self, promise: PromiseFuture<()>) {
        lock!(self.0).add(promise);
    }

    pub fn fuse(&mut self) {
        lock!(self.0).fuse();
    }
}
