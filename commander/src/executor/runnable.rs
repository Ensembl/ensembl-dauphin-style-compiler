use crate::executor::taskcontainer::TaskContainer;
use crate::executor::taskcontainerhandle::TaskContainerHandle;
use super::runqueue::RunQueue;

/* VERY HOT CODE PATH: BE EFFICIENT NOT PRETTY */

/* A Runnable contains a group of RunQueues. Each RunQueue has a different priority.
 * When asked to run a task, Runnable diverts the call to the RunQueue with the
 * highest priority.
 */

pub(super) struct Runnable {
    range: Option<(usize,usize)>, // ends are both INCLUSIVE
    queues: Vec<Option<RunQueue>>
}

impl Runnable {
    pub(super) fn new() -> Runnable {
        Runnable {
            range: None,
            queues: vec![]
        }
    }

    fn ensure(&mut self, index: usize) {
        if self.queues.len() <= index {
            self.queues.resize_with(index+1,Default::default)
        }
        if self.queues[index].is_none() {
            self.queues[index] = Some(RunQueue::new());
        }
    }

    fn update_limits_added(&mut self, index: usize) {
        if let Some((min,max)) = &mut self.range {
            if index < *min { *min = index; }
            if index > *max { *max = index; }
        } else {
            self.range = Some((index,index));
        }
    }

    fn find(&self, range: &mut dyn Iterator<Item=usize>, index: usize, cur_val: usize) -> Option<usize> {
        if index == cur_val {
            for i in range {
                if let Some(queue) = &self.queues[i] {
                    if !queue.empty() {
                        return Some(i);
                    }
                }
            }
            None
        } else {
            Some(cur_val)
        }
    }

    fn update_limits_removed(&mut self, index: usize) {
        let mut new_min = None;
        let mut new_max = None;
        if let Some((min,max)) = &self.range {
            if index != *min || index != *max {
                new_min = self.find(&mut (index..self.queues.len()),index,*min);
                new_max = self.find(&mut (0..index).rev(),index,*max);
            }
        }
        if let (Some(new_min),Some(new_max)) = (new_min,new_max) {
            self.range = Some((new_min,new_max));
        } else {
            self.range = None;
        }
    }

    pub(super) fn block(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            let queue = self.queues[index].as_mut().unwrap();
            queue.block(handle);
            if queue.empty() {
                self.update_limits_removed(index);
            }
        }
    }

    pub(super) fn unblock(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            self.queues[index].as_mut().unwrap().unblock(handle);
            self.update_limits_added(index);
        }
    }

    pub(super) fn add(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            self.ensure(index);
            self.queues[index].as_mut().unwrap().add(handle);
            self.update_limits_added(index);
        }
    }

    pub(super) fn remove(&mut self, tasks: &TaskContainer, handle: &TaskContainerHandle) {
        if let Some(task) = tasks.get(handle) {
            let index = task.get_priority() as usize;
            self.ensure(index);
            let queue = self.queues[index].as_mut().unwrap();
            queue.remove(handle);
            if queue.empty() {
                self.update_limits_removed(index);
            }
        }
    }

    pub(super) fn run(&mut self, tasks: &mut TaskContainer, tick_index: u64) -> bool {
        if let Some((index,_)) = self.range {
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
