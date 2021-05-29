use hashbrown::HashSet;
use crate::executor::taskcontainer::TaskContainer;
use crate::executor::taskcontainerhandle::{ TaskContainerHandle, TaskContainerHandleData };

/* A RunQueue contains a list of runnable tasks of the same priority. They are
 * run in order, once per call to run() with this struct remembering the next
 * task to run. It sits inside a Runnable which contains all the RunQueues of
 * different priorities. This, in turn, sits inside the excutor.
 */

struct RunQueueMember {
    index: usize,
    blocked: bool
}

pub(super) struct RunQueue {
    present: TaskContainerHandleData<RunQueueMember>,
    tasks: Vec<TaskContainerHandle>,
    next_task: usize,
    num_unblocked: usize
}

impl RunQueue {
    pub(super) fn new() -> RunQueue {
        RunQueue {
            present: TaskContainerHandleData::new(),
            tasks: Vec::new(),
            next_task: 0,
            num_unblocked: 0
        }
    }

    pub(super) fn empty(&self) -> bool { self.num_unblocked == 0 }

    pub(super) fn block(&mut self, handle: &TaskContainerHandle) {
        if let Some(member) = self.present.get_mut(handle) {
            if !member.blocked {
                member.blocked = true;
                self.num_unblocked -= 1;
            }
        }
    }

    pub(super) fn unblock(&mut self, handle: &TaskContainerHandle) {
        if let Some(member) = self.present.get_mut(handle) {
            if member.blocked {
                member.blocked = false;
                self.num_unblocked += 1;
            }
        }
    }

    pub(super) fn add(&mut self, handle: &TaskContainerHandle) {
        if self.present.get(handle).is_none() {
            let member = RunQueueMember {
                index: self.tasks.len(),
                blocked: false
            };
            self.present.insert(handle,member);
            self.tasks.push(handle.clone());
            self.num_unblocked += 1;
        }
    }

    pub(super) fn remove(&mut self, handle: &TaskContainerHandle) {
        if let Some(index) = self.tasks.iter().position(|h| h == handle) {
            if index < self.next_task {
                self.next_task -= 1;
            }
            self.num_unblocked -= 1;
            self.present.remove(handle);
            self.tasks.remove(index);
        }
    }

    fn next_task(&mut self) -> &TaskContainerHandle {
        loop {
            if self.next_task >= self.tasks.len() {
                self.next_task = 0;
            }
            self.next_task += 1;
            let handle = &self.tasks[self.next_task-1];
            if let Some(member) = self.present.get(handle) {
                if !member.blocked {
                    return handle;
                }
            }
        }
    }

    pub(super) fn run(&mut self, tasks: &mut TaskContainer, tick_index: u64) {
        if let Some(task) = tasks.get_mut(self.next_task()) {
            task.run(tick_index);
        }
    }
}

#[cfg(test)]
mod test {
    use crate::executor::taskcontainer::TaskContainer;
    use crate::task::faketask::FakeTask;
    use super::*;

    #[test]
    pub fn test_runqueue() {
        let mut tasks = TaskContainer::new();
        let mut q = RunQueue::new();
        /* test */
        assert!(q.empty());
        let h1 = tasks.allocate();
        let t1 = FakeTask::new(0);
        tasks.set(&h1,Box::new(t1.clone()));
        /* single task: check runs */
        q.add(&h1);
        assert!(!q.empty());
        assert_eq!(0,t1.run_count());
        q.run(&mut tasks,0);
        assert_eq!(1,t1.run_count());
        q.run(&mut tasks,0);
        assert_eq!(2,t1.run_count());
        /* add second and third task and check run fairly */
        let h2 = tasks.allocate();
        let t2 = FakeTask::new(0);
        tasks.set(&h2,Box::new(t2.clone()));
        let h3 = tasks.allocate();
        let t3 = FakeTask::new(0);
        tasks.set(&h3,Box::new(t3.clone()));
        q.add(&h2);
        q.add(&h3);
        q.run(&mut tasks,0);
        q.run(&mut tasks,0);
        assert_eq!(1,t2.run_count());
        assert_eq!(1,t3.run_count());
        q.run(&mut tasks,0);
        assert_eq!(3,t1.run_count());
        /* remove first and check for queue rewind */
        q.remove(&h1);
        q.run(&mut tasks,0);
        assert_eq!(2,t2.run_count());
        /* remove three and check for end-skip and no rewind */
        q.remove(&h3);
        q.run(&mut tasks,0);
        assert_eq!(3,t2.run_count());
        /* remove two to check for emptying */
        q.remove(&h2);
        assert!(q.empty());
    }
}