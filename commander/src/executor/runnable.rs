use std::collections::BTreeMap;
use crate::executor::taskcontainer::{ TaskContainer, TaskContainerHandle };
use super::runqueue::RunQueue;

/* VERY HOT CODE PATH: BEEFFICIENT NOT PRETTY */

/* A Runnable contains a group of RunQueues. Each RunQueue has a different priority.
 * When asked to run a task, Runnable diverts the call to the RunQueue with the
 * highest priority.
 */

pub(super) struct Runnable {
    first_used: Option<usize>,
    queues: Vec<Option<RunQueue>>
}

impl Runnable {
    pub(super) fn new() -> Runnable {
        Runnable {
            first_used: None,
            queues: vec![]
        }
    }

    fn ensure(&mut self, index: usize) {
        while self.queues.len() <= index {
            self.queues.push(None);
        }
        if self.queues[index].is_none() {
            self.queues[index] = Some(RunQueue::new());
        }
    }

    pub(super) fn add(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            self.ensure(index);
            self.queues[index].as_mut().unwrap().add(handle);
            if self.first_used.is_none() || self.first_used.unwrap() > index {
                self.first_used = Some(index);
            }
        }
    }

    pub(super) fn remove(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            self.ensure(index);
            let queue = self.queues[index].as_mut().unwrap();
            queue.remove(handle);
            if queue.empty() {
                self.queues[index] = None;
                self.first_used = None;
                for i in index..self.queues.len() {
                    if self.queues[i].is_some() {
                        self.first_used = Some(i);
                    }
                }
            }
        }
    }

    pub(super) fn run(&mut self, tasks: &mut TaskContainer, tick_index: u64) -> bool {
        if let Some(index) = self.first_used {
            self.queues[index].as_mut().unwrap().run(tasks,tick_index);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use crate::task::faketask::FakeTask;
    use super::*;

    #[test]
    pub fn test_runnable() {
        let mut tasks = TaskContainer::new();
        let mut r = Runnable::new();
        /* 1: h1, h2; 2: h3, 3: h4 */
        let h1 = tasks.allocate();
        let t1 = FakeTask::new(1);
        tasks.set(&h1,Box::new(t1.clone()));
        let h2 = tasks.allocate();
        let t2 = FakeTask::new(1);
        tasks.set(&h2,Box::new(t2.clone()));
        let h3 = tasks.allocate();
        let t3 = FakeTask::new(2);
        tasks.set(&h3,Box::new(t3.clone()));
        let h4 = tasks.allocate();
        let t4 = FakeTask::new(3);
        tasks.set(&h4,Box::new(t4.clone()));
        /* add all four and check just h1, h2 run */
        r.add(&mut tasks,&h1);
        r.add(&mut tasks,&h2);
        r.add(&mut tasks,&h3);
        r.add(&mut tasks,&h4);
        r.run(&mut tasks,0);
        r.run(&mut tasks,0);
        r.run(&mut tasks,0);
        assert_eq!(2,t1.run_count());
        assert_eq!(1,t2.run_count());
        assert_eq!(0,t3.run_count());
        assert_eq!(0,t4.run_count());
        /* remove h1 and check h2 just runs */
        r.remove(&mut tasks,&h1);
        r.run(&mut tasks,0);
        r.run(&mut tasks,0);
        assert_eq!(2,t1.run_count());
        assert_eq!(3,t2.run_count());
        assert_eq!(0,t3.run_count());
        assert_eq!(0,t4.run_count());
        /* remove h2 and check just h3 runs */
        r.remove(&mut tasks,&h2);
        r.run(&mut tasks,0);
        assert!(r.run(&mut tasks,0));
        assert_eq!(2,t1.run_count());
        assert_eq!(3,t2.run_count());
        assert_eq!(2,t3.run_count());
        assert_eq!(0,t4.run_count());
        /* remove h3 and h4 and check run returns false */
        r.remove(&mut tasks,&h3);
        r.remove(&mut tasks,&h4);
        assert!(!r.run(&mut tasks,0));
    }
}
