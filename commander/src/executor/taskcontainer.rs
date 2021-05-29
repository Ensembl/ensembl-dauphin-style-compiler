use crate::task::taskhandle::ExecutorTaskHandle;
use super::taskcontainerhandle::{ TaskContainerHandle, TaskContainerHandleData };

/* A TaskContainer is a place to store ExecutorTasks and give them a convenient
 * handle, a TaskContainerHandle. This can be passed around, cloned, etc, with
 * impunity. The executor can then extract the corresponding ExecutorTask. The
 * handle comprises a tuple. The first arg is the slot where the Task is/was stored,
 * the second a globally unique identifier to ensure that the handle is current.
 */

pub(crate) struct TaskContainer(TaskContainerHandleData<Box<dyn ExecutorTaskHandle>>);

impl TaskContainer {
    pub(crate) fn new() -> TaskContainer {
        TaskContainer(TaskContainerHandleData::new())
    }

    pub(crate) fn allocate(&mut self) -> TaskContainerHandle { self.0.allocate() }
    pub(super) fn all_handles(&self) -> Vec<TaskContainerHandle> { self.0.all_handles() }
    pub(crate) fn set(&mut self, handle: &TaskContainerHandle, task: Box<dyn ExecutorTaskHandle>) {
        self.0.set(handle,task);
    }

    pub(crate) fn remove(&mut self, handle: &TaskContainerHandle) {
        self.0.remove(handle);
    }

    pub(crate) fn get(&self, handle: &TaskContainerHandle) -> Option<&Box<dyn ExecutorTaskHandle>> { 
        self.0.get(handle)
    }

    pub(crate) fn get_mut(&mut self, handle: &TaskContainerHandle) -> Option<&mut Box<dyn ExecutorTaskHandle>> {
        self.0.get_mut(handle)
    }

    #[allow(unused)]
    pub(super) fn len(&self) -> usize { self.0.len() }
}
