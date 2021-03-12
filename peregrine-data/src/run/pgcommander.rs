use crate::lock;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use commander::{ RunSlot, Lock };

pub trait Commander {
    fn start(&self);
    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>);
    fn make_lock(&self) -> Lock;
}

pub struct PgCommanderTaskSpec<T> {
    pub name: String,
    pub prio: i8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub task: Pin<Box<dyn Future<Output=anyhow::Result<T>>>>
}

#[derive(Clone)]
pub struct PgCommander(Arc<Mutex<Box<dyn Commander>>>);

impl PgCommander {
    pub fn new(c: Box<dyn Commander>) -> PgCommander {
        PgCommander(Arc::new(Mutex::new(c)))
    }

    pub fn add_task(&self, t: PgCommanderTaskSpec<()>) {
        lock!(self.0).add_task(&t.name,t.prio,t.slot,t.timeout,t.task)
    }
}