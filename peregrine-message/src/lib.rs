use std::fmt;
use std::sync::{ Arc, Mutex };
use commander::FusePromise;

#[derive(Clone)]
pub struct Instigator<E>(FusePromise<Result<(),E>>,Arc<Mutex<i32>>) where E: Clone + 'static;

impl<E> Instigator<E> where E: Clone + 'static {
    pub fn new_with_fuse(fuse: FusePromise<Result<(),E>>) -> Instigator<E> {
        Instigator(fuse,Arc::new(Mutex::new(1)))
    }

    pub fn new() -> Instigator<E> {
        Self::new_with_fuse(FusePromise::new())
    }

    // TODO access fuse

    pub fn error(&mut self, error: E) {
        *self.1.lock().unwrap() = 0;
        self.0.fuse(Err(error));
    }

    pub fn done(&mut self) {
        let mut v = self.1.lock().unwrap();
        *v -= 1;
        if *v == 0 {
            *v = -1;
            self.0.fuse(Ok(()));
        }
    }

    pub fn merge<F,G>(&mut self, mut other: Instigator<G>, cb: F) where F: FnOnce(G) -> E + 'static, G: Clone {
        /* if other is already fused as successful then we can just ignore it as we aren't */
        if *other.1.lock().unwrap() != -1 {
            other.0.add_downstream(&self.0, |v| v.map_err(move |e| cb(e)));
            *self.1.lock().unwrap() += *other.1.lock().unwrap();
            other.1 = self.1.clone();
        }
    }
}

pub enum MessageLevel {
    Notice,
    Warn,
    Error
}

pub enum MessageCategory {
    BadFrontend,
    BadCode,
    BadData,
    BadBackend,
    BadInfrastructure,
    Unknown
}

pub trait PeregrineMessage : Send + Sync {
    fn level(&self) -> MessageLevel;
    fn category(&self) -> MessageCategory;
    fn now_unstable(&self) -> bool;
    fn degraded_experience(&self) -> bool;
    fn code(&self) -> (u64,u64);
    fn to_message_string(&self) -> String;
    fn cause_message(&self) -> Option<&(dyn PeregrineMessage + 'static)> { None }
}

impl fmt::Display for dyn PeregrineMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}

impl fmt::Debug for dyn PeregrineMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.to_message_string())
    }
}
