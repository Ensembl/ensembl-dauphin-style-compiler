use std::future::Future;
use std::pin::Pin;
use commander::{ Executor, RunSlot };
use owning_ref::MutexGuardRefMut;

pub trait PgCommander {
    fn start(&self);
    fn executor(&self) -> anyhow::Result<MutexGuardRefMut<Executor>>;
    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>);
}