use std::collections::{ VecDeque, HashMap, HashSet };
use super::taskcontainerhandle::TaskContainerHandle;

// TODO use keyed

#[derive(Clone)]
pub struct Lock(u64);

impl Lock {
    pub(crate) fn make_guard<'t,F>(&self, cb: F) -> LockGuard<'t> where F: FnOnce(&Lock) + 't { LockGuard(self.clone(),Some(Box::new(cb))) }
}

pub struct LockGuard<'t>(Lock,Option<Box<dyn FnOnce(&Lock) + 't>>);

impl<'t> Drop for LockGuard<'t> {
    fn drop(&mut self) {
        self.1.take().unwrap()(&self.0)
    }
}

pub(super) struct LockManager {
    tasks: HashMap<u64,VecDeque<TaskContainerHandle>>,
    blocked: HashSet<TaskContainerHandle>,
    callbacks: HashMap<TaskContainerHandle,Box<dyn FnOnce()>>,
    next_id: u64
}

impl LockManager {
    pub(super) fn new() -> LockManager {
        LockManager {
            tasks: HashMap::new(),
            blocked: HashSet::new(),
            callbacks: HashMap::new(),
            next_id: 0
        }
    }

    pub(super) fn make_lock(&mut self) -> Lock {
        self.next_id += 1;
        Lock(self.next_id)
    }

    pub(super) fn lock<F>(&mut self, task: &TaskContainerHandle, lock: Lock, cb: F) where F: FnOnce() + 'static {
        if let Some(ref mut waiting) = self.tasks.get_mut(&lock.0) {
            waiting.push_back(task.clone());
            self.blocked.insert(task.clone());
            self.callbacks.insert(task.clone(),Box::new(cb));
        } else {
            self.tasks.insert(lock.0,VecDeque::new());
            cb();
        }
    }

    pub(super) fn unlock(&mut self, lock: Lock) {
        let waiting = self.tasks.get_mut(&lock.0).unwrap();
        if let Some(unblocked) = waiting.pop_front() {
            self.blocked.remove(&unblocked);
            if let Some(cb) = self.callbacks.remove(&unblocked) {
                cb();
            }
        } else {
            self.tasks.remove(&lock.0);
        }
    }
}

#[cfg(test)]
mod test {
    use blackbox::*;
    use futures::future;
    use std::collections::HashSet;
    use std::sync::{ Arc, Mutex };
    use crate::{cdr_lock, cdr_tick, corefutures::promisefuture::PromiseFuture};
    use crate::{ Executor, RunConfig };
    use crate::integration::integration::SleepQuantity;
    use crate::integration::testintegration::{ TestIntegration, tick_helper };
    use crate::task::task::{ KillReason, TaskResult };
    use super::*;
    
    #[test]
    pub fn test_lock_smoke() {
        let integration = TestIntegration::new();
        let mut x = Executor::new(integration.clone());
        let cfg = RunConfig::new(None,2,None);
        let cfg2 = RunConfig::new(None,3,None);
        let agent = x.new_agent(&cfg,"test");
        let agent2 = x.new_agent(&cfg2,"test2");
        let lock = x.make_lock();
        let lock2 = lock.clone();
        let report = Arc::new(Mutex::new(vec![]));
        let report2 = report.clone();
        let report3 = report.clone();
        let step = async move {
            let guard = cdr_lock(&lock).await;
            report2.lock().unwrap().push("A");
            cdr_tick(5).await;
            report2.lock().unwrap().push("B");
            drop(guard);
            cdr_tick(1).await;
            report2.lock().unwrap().push("F");
        };
        let step2 = async move {
            report3.lock().unwrap().push("C");
            cdr_tick(1).await;
            let guard = cdr_lock(&lock2).await;
            report3.lock().unwrap().push("D");
            cdr_tick(1).await;
            report3.lock().unwrap().push("E");
        };
        let handle = x.add(step,agent);
        let handle2 = x.add(step2,agent2);
        for _ in 0..10 {
            report.lock().unwrap().push(".");
            x.tick(1.);
        }
        assert_eq!(".AC.....BD.FE...",report.lock().unwrap().join(""));
    }
}
