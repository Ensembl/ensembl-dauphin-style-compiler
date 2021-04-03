use std::rc::Rc;
use std::sync::Mutex;
use commander::PromiseFuture;

struct RedrawNeededData {
    edge: bool,
    locks: i32,
    promises: Vec<PromiseFuture<()>>
}

impl RedrawNeededData {
    fn new() -> RedrawNeededData {
        RedrawNeededData {
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

    fn maybe_needed(&mut self) -> Option<PromiseFuture<()>> {
        if self.edge || self.locks > 0 { 
            self.edge = false;
            return None;
        }
        let promise = PromiseFuture::new();
        self.promises.push(promise.clone());
        return Some(promise);
    }
}

#[derive(Clone)]
pub struct RedrawNeeded(Rc<Mutex<RedrawNeededData>>);

#[derive(Clone)]
pub struct RedrawNeededLock(RedrawNeeded);

impl<'t> Drop for RedrawNeededLock {
    fn drop(&mut self) {
        self.0.delta(-1);
    }
}

impl RedrawNeeded {
    pub fn new() -> RedrawNeeded {
        RedrawNeeded(Rc::new(Mutex::new(RedrawNeededData::new())))
    }

    fn delta(&self, d: i32) {
        self.0.lock().unwrap().delta(d);
    }
    
    pub fn lock(&self) -> RedrawNeededLock {
        self.delta(1);
        RedrawNeededLock(self.clone())
    }

    pub fn set(&self) {
        self.0.lock().unwrap().set();
    }

    pub async fn wait_until_redraw_needed(&self) {
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
