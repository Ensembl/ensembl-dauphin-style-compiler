use binary_heap_plus::{ BinaryHeap, MinComparator };
use std::default::Default;

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub(crate) struct TaskContainerHandle(usize,u64); /* (slot,identity) */

impl TaskContainerHandle {
    pub(crate) fn identity(&self) -> u64 { self.1 }
}

pub(crate) struct TaskContainerHandleData<D> {
    free_slots: BinaryHeap<usize,MinComparator>,
    tasks: Vec<Option<(D,u64)>>,
    identity: u64
}

impl<D> TaskContainerHandleData<D> {
    pub(crate) fn new() -> TaskContainerHandleData<D> {
        TaskContainerHandleData {
            free_slots: BinaryHeap::new_min(),
            tasks: Vec::new(),
            identity: 2
        }
    }

    pub(crate) fn allocate(&mut self) -> TaskContainerHandle {
        let slot = self.free_slots.pop().unwrap_or_else(|| {
            self.tasks.push(None);
            self.tasks.len()-1
        });
        self.tasks[slot] = None;
        let out = TaskContainerHandle(slot,self.identity);
        self.identity += 1;
        out
    }

    fn extend(&mut self, handle: &TaskContainerHandle) {
        if self.tasks.len() <= handle.0 {
            self.tasks.resize_with(handle.0+1, Default::default);
        }
    }

    pub(super) fn all_handles(&self) -> Vec<TaskContainerHandle> {
        self.tasks.iter().enumerate()
            .filter(|x| x.1.is_some())
            .map(|(i,x)| TaskContainerHandle(i,x.as_ref().unwrap().1))
            .collect()
    }

    pub(crate) fn insert(&mut self, handle: &TaskContainerHandle, task: D) {
        self.extend(handle);
        self.tasks[handle.0] = Some((task,handle.1));
    }

    pub(crate) fn set(&mut self, handle: &TaskContainerHandle, task: D) {
        self.tasks[handle.0] = Some((task,handle.1));
    }

    pub(crate) fn remove(&mut self, handle: &TaskContainerHandle) {
        if self.get(handle).is_none() { return; }
        self.tasks[handle.0] = None;
        self.free_slots.push(handle.0);
    }

    pub(crate) fn get(&self, handle: &TaskContainerHandle) -> Option<&D> { 
        match self.tasks.get(handle.0) {
            Some(Some((task,identity))) if handle.1 == *identity => {
                Some(task)
            },
            _ => None
        }
    }

    pub(crate) fn get_mut(&mut self, handle: &TaskContainerHandle) -> Option<&mut D> {
        match self.tasks.get_mut(handle.0) {
            Some(Some((task,identity))) if handle.1 == *identity => {
                Some(task)
            },
            _ => None
        }
    }

    #[allow(unused)]
    pub(super) fn len(&self) -> usize { self.tasks.len() - self.free_slots.len() }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_container() {
        let mut tasks = TaskContainerHandleData::<u8>::new();
        assert!(tasks.tasks.len()==0);
        assert!(tasks.free_slots.len()==0);
        let h1 = tasks.allocate();
        tasks.set(&h1,1);
        let h2 = tasks.allocate();
        tasks.set(&h2,2);
        let h3 = tasks.allocate();
        tasks.set(&h3,3);
        assert_eq!(0,h1.0);
        assert_eq!(1,h2.0);
        assert_eq!(2,h3.0);
        tasks.remove(&h2);
        assert!(tasks.free_slots.len()==1);
        assert_eq!(1,*tasks.free_slots.peek().unwrap());
        tasks.remove(&h1);
        assert!(tasks.free_slots.len()==2);
        assert_eq!(0,*tasks.free_slots.peek().unwrap());
        let h4 = tasks.allocate();
        tasks.set(&h4,4);
        assert_eq!(0,h4.0);
        assert!(tasks.free_slots.len()==1);
        assert_eq!(1,*tasks.free_slots.peek().unwrap());
    }

    #[test]
    pub fn test_expired_handles() {
        let mut tasks = TaskContainerHandleData::<u8>::new();
        assert!(tasks.tasks.len()==0);
        assert!(tasks.free_slots.len()==0);
        /* h1 => slot 0, h2 => slot 1 */
        let h1 = tasks.allocate();
        let h2 = tasks.allocate();
        tasks.set(&h1,1);
        tasks.set(&h2,2);
        assert_eq!(0,h1.0);
        assert_eq!(1,h2.0);
        assert!(tasks.get(&h1).is_some());
        assert!(tasks.get(&h2).is_some());
        /* remove h1, freeing slot 0 */
        tasks.remove(&h1);
        /* allocate t3/h3 in h1's place */
        let h3 = tasks.allocate();
        tasks.set(&h3,3);
        assert_eq!(0,h3.0);
        assert!(tasks.get(&h1).is_none());
        assert!(tasks.get(&h3).is_some());
        /* verify double removal does nothing */
        tasks.remove(&h1);
        assert!(tasks.get(&h3).is_some());
    }
}
