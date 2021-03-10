use crate::lock;
use std::sync::{ Arc, Mutex };
use super::fuse::FusePromise;
use commander::PromiseFuture;

#[derive(Clone)]
pub struct LockPromise(Arc<Mutex<(u64,FusePromise<()>)>>);

impl LockPromise {
    pub fn new(fp: &FusePromise<()>) -> LockPromise {
        LockPromise(Arc::new(Mutex::new((1,fp.clone()))))
    }

    pub fn lock(&self) {
        lock!(self.0).0 += 1;
    }

    pub fn unlock(&self) {
        let mut fuse = None;
        let mut v = lock!(self.0);
        v.0 -= 1;
        if v.0 == 0 {
            fuse = Some(v.1.clone());
        }
        drop(v);
        if let Some(mut fuse) = fuse {
            fuse.fuse(());
        }
    }
}

pub struct CountingPromiseData {
    lock: LockPromise,
    fuse: FusePromise<()>
}

#[derive(Clone)]
pub struct CountingPromise(Arc<Mutex<CountingPromiseData>>);

impl CountingPromise {
    pub fn new() -> CountingPromise {
        let fuse = FusePromise::new();
        let lock = LockPromise::new(&fuse);
        CountingPromise(Arc::new(Mutex::new(CountingPromiseData { lock, fuse })))
    }

    pub fn lock(&self) {
        lock!(self.0).lock.lock();
    }

    pub fn unlock(&self) {
        lock!(self.0).lock.unlock();
    }

    pub async fn wait(&self) {
        let p = PromiseFuture::new();
        lock!(self.0).fuse.add(p.clone());
        p.await;
    }
}
