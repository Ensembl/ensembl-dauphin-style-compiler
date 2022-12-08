use std::rc::Rc;
use std::sync::{Mutex, Arc};
use commander::PromiseFuture;
use peregrine_toolkit::{lock};

struct NeededData {
    edge: bool,
    locks: i32,
    promises: Vec<PromiseFuture<()>>
}

impl NeededData {
    fn new() -> NeededData {
        NeededData {
            edge: false,
            locks: 0,
            promises: vec![]
        }
    }    

    fn delta(&mut self, d: i32) {
        self.locks += d;
        if self.edge || self.locks>0 {
            for p in self.promises.drain(..) {
                p.satisfy(());
            }
        }
    }

    fn set(&mut self) {
        self.edge = true;
        for p in self.promises.drain(..) {
            p.satisfy(());
        }
    }

    fn is_needed(&mut self) -> bool {
        if self.edge || self.locks > 0 { 
            self.edge = false;
            true
        } else {
            false
        }
    }

    fn maybe_needed(&mut self) -> Option<PromiseFuture<()>> {
        if self.is_needed() {
            self.edge = false;
            return None;
        }
        let promise = PromiseFuture::new();
        self.promises.push(promise.clone());
        return Some(promise);
    }
}

#[derive(Clone)]
pub struct Needed(Rc<Mutex<NeededData>>);

pub struct NeededLock(Needed);

impl<'t> Drop for NeededLock {
    fn drop(&mut self) {
        self.0.delta(-1);
    }
}

impl Needed {
    pub fn new() -> Needed {
        Needed(Rc::new(Mutex::new(NeededData::new())))
    }

    fn delta(&self, d: i32) {
        lock!(self.0).delta(d);
    }
    
    pub fn lock(&self) -> NeededLock {
        self.delta(1);
        NeededLock(self.clone())
    }

    pub fn needed_on_drop(&self) -> NeededOnDrop {
        NeededOnDrop(Arc::new(NeededOnDropInternal(self.clone())))
    }

    pub fn set(&self) {
        lock!(self.0).set();
    }

    pub fn is_needed(&self) -> bool {
        lock!(self.0).is_needed()
    }

    pub async fn wait_until_needed(&self) {
        loop {
            let mut r = self.0.lock().unwrap();
            let promise = r.maybe_needed();
            drop(r);
            if let Some(promise) = promise {
                promise.await;
            } else {
                return;
            }
        }
    }
}

struct NeededOnDropInternal(Needed);

impl Drop for NeededOnDropInternal {
    fn drop(&mut self) { self.0.set(); }
}

#[derive(Clone)]
pub struct NeededOnDrop(Arc<NeededOnDropInternal>);
