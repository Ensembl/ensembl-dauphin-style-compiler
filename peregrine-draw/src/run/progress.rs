use std::{future::Future };
use std::pin::Pin;
use commander::{ FusePromise, PromiseFuture };
use crate::util::message::Message;
use peregrine_message::Instigator;

pub struct Progress(FusePromise<Result<(),Message>>);

impl Progress {
    pub(crate) fn new() -> (Progress,Instigator<Message>) {
        let fuse = FusePromise::new();
        (Progress(fuse.clone()),Instigator::new_with_fuse(fuse))
    }

    pub fn waiter(&mut self) -> Pin<Box<dyn Future<Output=Result<(),Message>>>> {
        let p = PromiseFuture::new();
        self.0.add(p.clone());
        Box::pin(p)
    }

    pub async fn wait(&mut self) -> Result<(),Message> {
        self.waiter().await
    }

    pub fn add_callback<F>(&mut self, cb: F) where F: FnOnce(Result<(),Message>) + 'static {
        self.0.add_callback(cb);
    }

    pub fn new_merged(&self, other: Progress) -> Progress {
        let fuse = FusePromise::new();
        self.0.add_downstream(&fuse,|e| e);
        other.0.add_downstream(&fuse,|e| e);
        Progress(fuse)
    }
}
