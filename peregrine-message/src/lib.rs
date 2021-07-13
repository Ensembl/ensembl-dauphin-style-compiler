use std::fmt;
use std::sync::{ Arc, Mutex };
use commander::SendFusePromise;

#[derive(Clone)]
pub struct Instigator<E>(SendFusePromise<Result<(),E>>,Arc<Mutex<i32>>) where E: Clone + Send + 'static;

impl<E> Instigator<E> where E: Clone + 'static + Send {
    pub fn new_with_fuse(fuse: SendFusePromise<Result<(),E>>) -> Instigator<E> {
        Instigator(fuse,Arc::new(Mutex::new(1)))
    }

    pub fn new() -> Instigator<E> {
        Self::new_with_fuse(SendFusePromise::new())
    }

    // TODO access fuse

    pub fn error(&self, error: E) {
        *self.1.lock().unwrap() = 0;
        self.0.fuse(Err(error));
    }

    pub fn done(&self) {
        let mut v = self.1.lock().unwrap();
        *v -= 1;
        if *v == 0 {
            *v = -1;
            self.0.fuse(Ok(()));
        }
    }

    pub fn merge<F,G>(&self, mut other: Instigator<G>, cb: F) where F: FnOnce(G) -> E + 'static + Send+Sync, G: Clone + Send {
        /* if other is already fused as successful then we can just ignore it as we aren't */
        if *other.1.lock().unwrap() != -1 {
            other.0.add_downstream(&self.0, |v| v.map_err(move |e| cb(e)));
            *self.1.lock().unwrap() += *other.1.lock().unwrap();
            other.1 = self.1.clone();
        }
    }

    pub fn to_reporter(self) -> Reporter<E> { Reporter(Arc::new(Mutex::new(ReporterDropper(self)))) }
}

struct ReporterDropper<E>(Instigator<E>) where E: Clone + 'static + Send;

impl<E> Drop for ReporterDropper<E> where E: Clone + 'static + Send {
    fn drop(&mut self) {
        self.0.done();
    }
}

#[derive(Clone)]
pub struct Reporter<E>(Arc<Mutex<ReporterDropper<E>>>) where E: Clone + 'static + Send;

impl<E> Reporter<E> where E: Clone + 'static + Send {
    pub fn error(&self, error: E) { self.0.lock().unwrap().0.error(error); }
}

pub enum MessageKind {
    Error,
    Interface
}

pub enum MessageAction {
    ImmediateRetry, /* just hit reload */
    RetrySoon,      /* hand on a bit and try again */
    OurMistake,     /* we messed up */
    YourMistake,    /* there's something wrong with your computer */
    Advisory        /* FYI (porbably drop these except in dev builds) */
}

pub enum MessageLikelihood {
    Inevitable,   /* network errors, etc... */
    Quality,      /* we should have done better, but didn't */
    Unlikely,     /* these errors shouldn't happen, but it's clear what did */
    Inconceivable /* how on earth did you get _that_ to happen? */
}

pub trait PeregrineMessage : Send + Sync {
    fn kind(&self) -> MessageKind;
    fn action(&self) -> MessageAction;
    fn likelihood(&self) -> MessageLikelihood;
    fn knock_on(&self) -> bool;
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
