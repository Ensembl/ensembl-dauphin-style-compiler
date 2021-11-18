use std::{future::Future, pin::Pin, sync::{Arc, Mutex}};
use commander::{FusePromise, PromiseFuture};
use crate::lock;

enum AsyncOnceData<T: Clone> {
    Unstarted(Option<Pin<Box<dyn Future<Output=T> + 'static>>>),
    Started(FusePromise<T>),
    Finished(T)
}

impl<T: Clone> AsyncOnceData<T> {
    fn start(&mut self) -> Option<(Pin<Box<dyn Future<Output=T> + 'static>>,FusePromise<T>)> {
        let mut out = None;
        let mut pending_cb = None;
        if let AsyncOnceData::Unstarted(ref mut cb) = self {
            pending_cb = cb.take();
        }
        if let Some(cb) = pending_cb {
            let fuse = FusePromise::new();
            *self = AsyncOnceData::Started(fuse.clone());    
            out = Some((cb,fuse));
        }
        out
    }
}

#[derive(Clone)]
pub struct AsyncOnce<T: Clone>(Arc<Mutex<AsyncOnceData<T>>>);

impl<T: Clone> AsyncOnce<T> {
    pub fn new<F>(cb: F) -> AsyncOnce<T> where F: Future<Output=T> + 'static {
        AsyncOnce(Arc::new(Mutex::new(AsyncOnceData::Unstarted(Some(Box::pin(cb))))))
    }

    pub async fn get(&self) -> T {
        let values = lock!(self.0).start();
        if let Some((cb,fuse)) = values {
            *lock!(self.0) = AsyncOnceData::Started(fuse.clone());
            let value = cb.await;
            *lock!(self.0) = AsyncOnceData::Finished(value.clone());
            fuse.fuse(value);
        }
        /* By this point, we know that we aren't AsyncOnceData::Unstarted */
        let mut pending_promise = None;
        if let AsyncOnceData::Started(fuse) = &*lock!(self.0) {
            let promise = PromiseFuture::new();
            fuse.add(promise.clone());
            pending_promise = Some(promise.clone());
        }
        if let Some(promise) = pending_promise {
            return promise.await;
        }
        /* By this point, we know we are AsyncOnceData::Finished */
        if let AsyncOnceData::Finished(value) = &*lock!(self.0) {
            return value.clone();
        } else {
            panic!("code invariant violated in AsyncOnce");
        }
    }

    pub fn peek(&self) -> Option<T> {
        match &*lock!(self.0) {
            AsyncOnceData::Finished(value) => {
                Some(value.clone())
            },
            _ => None
        }
    }
}
