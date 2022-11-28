use std::rc::Rc;
use std::sync::Mutex;
use commander::PromiseFuture;
use peregrine_toolkit::lock;

struct BlockerData {
    locks: i32,
    freewheel: bool,
    promises: Vec<PromiseFuture<()>>
}

// XXX race when not single threaded
impl BlockerData {
    fn new() -> BlockerData {
        BlockerData {
            locks: 0,
            freewheel: false,
            promises: vec![]
        }
    }    

    fn set_freewheel(&mut self, yn: bool) {
        self.freewheel = yn;
    }

    fn delta(&mut self, d: i32) {
        self.locks += d;
        if self.locks == 0 {
            for p in self.promises.drain(..) {
                p.satisfy(());
            }
        }
    }

    fn is_blocked(&mut self) -> bool {
        self.locks != 0 && !self.freewheel
    }

    fn maybe_blocked(&mut self) -> Option<PromiseFuture<()>> {
        if !self.is_blocked() {
            return None;
        }
        let promise = PromiseFuture::new();
        self.promises.push(promise.clone());
        return Some(promise);
    }
}

#[derive(Clone)]
pub struct Blocker(Rc<Mutex<BlockerData>>);

pub struct Lockout(Blocker);

impl<'t> Drop for Lockout {
    fn drop(&mut self) {
        self.0.delta(-1);
    }
}

impl Blocker {
    pub fn new() -> Blocker {
        Blocker(Rc::new(Mutex::new(BlockerData::new())))
    }

    fn delta(&self, d: i32) {
        self.0.lock().unwrap().delta(d);
    }
    
    pub fn set_freewheel(&self, yn: bool) {
        self.0.lock().unwrap().set_freewheel(yn);
    }

    pub fn lock(&self) -> Lockout {
        self.delta(1);
        Lockout(self.clone())
    }

    pub fn is_blocked(&mut self) -> bool {
        self.0.lock().unwrap().is_blocked()
    }

    pub async fn wait(&self) {
        loop {
            let mut r = lock!(self.0);
            let promise = r.maybe_blocked();
            drop(r);
            if let Some(promise) = promise {
                promise.await;
            } else {
                return;
            }
        }
    }
}

pub struct LockoutBool {
    blocker: Blocker,
    #[allow(unused)]
    lockout: Mutex<Option<Lockout>>
}

impl LockoutBool {
    pub fn new(blocker: &Blocker) -> LockoutBool {
        LockoutBool {
            blocker: blocker.clone(),
            lockout: Mutex::new(None),
        }
    }

    pub fn set(&self, yn: bool) {
        let mut lockout = lock!(self.lockout);
        if yn {
            if lockout.is_none() {
                *lockout = Some(self.blocker.lock());
            }
        } else {
            *lockout = None;
        }
    }
}
