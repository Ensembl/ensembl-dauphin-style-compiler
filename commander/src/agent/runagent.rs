use std::future::Future;
use crate::corefutures::promisefuture::PromiseFuture;
use crate::executor::lock::{ Lock, LockGuard };
use crate::executor::action::Action;
use crate::executor::link::TaskLink;
use crate::executor::request::Request;
use crate::executor::taskcontainerhandle::TaskContainerHandle;
use crate::integration::reentering::ReenteringIntegration;
use crate::task::runconfig::RunConfig;
use crate::task::taskhandle::TaskHandle;
use super::agent::Agent;

/* RunAgent is the Agent mixin responsible for various odds and ends around step execution */

pub(crate) struct RunAgent {
    tick_index: u64,
    config: RunConfig,
    integration: ReenteringIntegration,
    task_action_link: TaskLink<Action>,
    task_request_link: TaskLink<Request>,
    id: Option<(u64,u64)>,
    start_time: Option<f64>,
    elapsed: f64,
    run: f64,
    stats: bool
}

impl RunAgent {
    pub(super) fn new(integration: &ReenteringIntegration, task_action_link: &TaskLink<Action>, 
                        task_request_link: &TaskLink<Request>, config: &RunConfig) -> RunAgent {
        RunAgent {
            tick_index: 0,
            config: config.clone(),
            integration: integration.clone(),
            task_action_link: task_action_link.clone(),
            task_request_link: task_request_link.clone(),
            id: None,
            start_time: None,
            elapsed: 0.,
            run: 0.,
            stats: false
        }
    }

    pub(super) fn enable_stats(&mut self) {
        self.stats = true;
    }

    pub(super) fn set_tick_index(&mut self, tick: u64) {
        self.tick_index = tick;
    }

    pub(super) fn get_id(&self) -> Option<(u64,u64)> { self.id }
    pub(super) fn get_tick_index(&self) -> u64 { self.tick_index }
    pub(super) fn get_current_time(&self) -> f64 { self.integration.current_time() }
    pub(super) fn get_config(&self) -> &RunConfig { &self.config }

    pub(super) fn new_agent(&self, name: &str, rc: Option<RunConfig>) -> Agent {
        let rc = rc.unwrap_or(self.config.clone());
        Agent::new(&rc,&self.task_action_link.get_link(),&self.task_request_link.get_link(),&self.integration,name)
    }

    pub(super) fn submit<R,T>(&self, mut agent2: Agent, future: T) -> TaskHandle<R> where T: Future<Output=R> + 'static, R: 'static {
        let handle2 = TaskHandle::new(&mut agent2,Box::pin(future));
        self.task_request_link.add(Request::Create(Box::new(handle2.clone()),agent2.clone()));
        handle2
    }

    pub(super) fn add_timer<T>(&self, timeout: f64, callback: T) where T: FnOnce() + 'static {
        self.task_request_link.add(Request::Timer(timeout,Box::new(callback)));
    }

    pub(super) fn add_ticks_timer<T>(&self, ticks: u64, callback: T) where T: FnOnce() + 'static {
        self.task_request_link.add(Request::Tick(self.tick_index+ticks,Box::new(callback)));
    }

    pub(crate) fn register(&mut self, task_handle: &TaskContainerHandle, id: (u64,u64)) {
        self.task_action_link.register(task_handle);
        self.task_request_link.register(task_handle);
        self.id = Some(id);
    }

    pub(crate) fn lock(&self, lock: &Lock) -> impl Future<Output=LockGuard<'static>> {
        let promise = PromiseFuture::new();
        let promise2 = promise.clone();
        let lock2 = lock.clone();
        let task_request_link = self.task_request_link.clone();
        /* we ask the executor to lock this lock and call Callback A when that's done */
        self.task_request_link.add(Request::Lock(lock.clone(),Box::new(move || {
            /* Callback A: called when LOCKED ... */
            /* ... crate a guard to return so that we can unlock when we drop, which is when Callback B is called */
            let guard = lock2.make_guard(move |lock| {
                /* Callback B: the guard has been dropped, so tell the executor */
                task_request_link.add(Request::Unlock(lock.clone()));
            });
            /* ... and let the new locker continue */
            promise2.satisfy(guard);
        })));
        promise
    }

    pub(super) fn stats_time(&self) -> Option<f64> {
        if self.stats { Some(self.get_current_time()) } else { None }
    }

    pub(super) fn timing(&mut self, start: Option<f64>, end: Option<f64>) {
        if let (Some(start),Some(end)) = (start,end) {
            self.run += end-start;
            if self.start_time.is_none() { self.start_time = Some(start); }
            self.elapsed = end-self.start_time.unwrap();
        }
    }

    pub(super) fn clock_time(&self) -> f64 { self.elapsed }
    pub(super) fn run_time(&self) -> f64 { self.run }
    pub(super) fn state_enabled(&self) -> bool { self.stats }
}

#[cfg(test)]
mod test {
    use std::sync::{ Arc, Mutex };
    use crate::executor::executor::Executor;
    use crate::integration::testintegration::TestIntegration;
    use crate::task::task::KillReason;
    use super::*;

    #[test]
    pub fn test_create_subtask() {
        let integration = TestIntegration::new();
        let mut x = Executor::new(integration.clone());
        let cfg = RunConfig::new(None,3,None);
        let agent = x.new_agent(&cfg,"test");
        let agent2 = agent.clone();
        let tidied = Arc::new(Mutex::new(false));
        let tidied2 = tidied.clone();
        let step = async move {
            let agentb = agent2.new_agent(None,"task2");
            let agentb2 = agentb.clone();
            agent2.add(async move {
                agentb2.tick(1).await;
                *tidied2.lock().unwrap() = true;
                agentb2.tick(1).await;
            },agentb);
            42
        };
        let handle = x.add(step,agent);
        x.tick(1.);
        assert!(!*tidied.lock().unwrap());
        assert_eq!(Some(42),handle.take_result());
        x.tick(1.);
        assert!(*tidied.lock().unwrap());
        let all = x.summarize_all();
        assert_eq!(1,all.len());
        assert_eq!("task2",all[0].get_name());
    }

    #[test]
    pub fn test_kill_before_run() {
        let integration = TestIntegration::new();
        let mut x = Executor::new(integration.clone());
        let cfg = RunConfig::new(None,3,None);
        let agent = x.new_agent(&cfg,"test");
        let agent2 = agent.clone();
        let tidied = Arc::new(Mutex::new(false));
        let tidied2 = tidied.clone();
        let step = async move {
            let agentb = agent2.new_agent(None,"task2");
            agentb.finish(KillReason::Cancelled);
            let agentb2 = agentb.clone();
            agent2.add(async move {
                agentb2.tick(1).await;
                *tidied2.lock().unwrap() = true;
                agentb2.tick(1).await;
            },agentb);
            42
        };
        x.add(step,agent);
        assert_eq!(1,x.get_tasks().summarize_all().len());
    }
}
