use std::sync::{Arc, Mutex};
use commander::{FusePromise, PromiseFuture};

#[derive(Clone)]
pub(crate) struct BusyWaiter {
    fuse: Arc<Mutex<Option<FusePromise<()>>>>
}

impl BusyWaiter {
    pub(crate) fn new() -> BusyWaiter {
        BusyWaiter {
            fuse: Arc::new(Mutex::new(None))
        }
    }

    pub(crate) fn set(&self, yn: bool) {
        let mut fuse = self.fuse.lock().unwrap();
        if yn {
            if fuse.is_none() {
                *fuse = Some(FusePromise::new());
            }
        } else {
            if let Some(fuse) = fuse.as_mut() {
                fuse.fuse(());
            }
            *fuse = None;
        }
    }

    pub(crate) fn waiter(&self) -> PromiseFuture<()> {
        let p = PromiseFuture::new();
        if let Some(fuse) = self.fuse.lock().unwrap().as_mut() {
            fuse.add(p.clone());
        } else {
            p.satisfy(());
        }
        p
    }

    pub(crate) async fn wait(&self) {
        self.waiter().await
    }
}
