use crate::agent::agent::Agent;
use crate::task::taskhandle::ExecutorTaskHandle;
use super::lock::Lock;

pub(crate) enum Request {
    Create(Box<dyn ExecutorTaskHandle + 'static>,Agent),
    Tick(u64,Box<dyn FnOnce() + 'static>),
    Timer(f64,Box<dyn FnOnce() + 'static>),
    Lock(Lock,Box<dyn FnOnce() + 'static>),
    Unlock(Lock)
}
