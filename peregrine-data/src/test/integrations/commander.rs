use anyhow::Context;
use std::pin::Pin;
use std::future::Future;
use std::sync::{ Arc, Mutex, MutexGuard };
use commander::{ Integration, Lock, SleepQuantity, Executor, RunSlot, RunConfig, cdr_get_name, TaskHandle, TaskResult, cdr_in_agent, cdr_new_agent, cdr_add };
use crate::Commander;
use crate::util::message::DataMessage;

#[derive(Clone)]
pub struct TestCommanderIntegration {
    now: Arc<Mutex<f64>>
}

impl TestCommanderIntegration {
    pub(crate) fn new() -> TestCommanderIntegration {
        TestCommanderIntegration {
            now: Arc::new(Mutex::new(0.))
        }
    }

    pub(crate) fn add_time(&self, time: f64) {
        *self.now.lock().unwrap() += time;
    }
}

impl Integration for TestCommanderIntegration {
    fn current_time(&self) -> f64 {
        *self.now.lock().unwrap()
    }

    fn sleep(&self, _amount: SleepQuantity) {}
}

#[derive(Clone)]
pub struct TestCommander {
    integration: TestCommanderIntegration,
    executor: Arc<Mutex<Executor>>
}

impl TestCommander {
    pub(crate) fn new() -> TestCommander {
        let integration = TestCommanderIntegration::new();
        TestCommander {
            integration: integration.clone(),
            executor: Arc::new(Mutex::new(Executor::new(integration)))
        }
    }

    pub(crate) fn add_time(&self, time: f64) {
        self.integration.add_time(time);
    }

    pub(crate) fn tick(&self) {
        self.executor.lock().unwrap().tick(1.);
        self.add_time(1.);
    }
}

/*
pub(crate) fn console_warn(console: &TestConsole, e: anyhow::Result<()>) {
    match e {
        Ok(e) => e,
        Err(e) => {
            console.message(&format!("{:?}",e));
        }
    }
}

async fn catch_errors(console: TestConsole, f: Pin<Box<dyn Future<Output=Result<(),DataMessage>>>>) {
    console_warn(&console,f.await.with_context(|| format!("async: {}",cdr_get_name())));
}

async fn finish(console: TestConsole, res: TaskHandle<()>, name: String) {
    res.finish_future().await;
    match res.task_state() {
        TaskResult::Killed(reason) => {
            console.message(&format!("async {}: {}",reason,name));
        },
        _ => {}
    }
}
*/

impl Commander for TestCommander {
    fn start(&self) {
    }

    fn executor(&self) -> MutexGuard<Executor> { self.executor.lock().unwrap() }

    fn identity(&self) -> u64 { 0 }

    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=Result<(),DataMessage>> + 'static>>) -> TaskHandle<Result<(),DataMessage>> {
        let rc = RunConfig::new(slot,prio,timeout);
        let rc2 = RunConfig::new(None,prio,None);
        if cdr_in_agent() {
            let agent = cdr_new_agent(Some(rc),name);
            cdr_add(Box::pin(f),agent)
        } else {
            let mut exe = self.executor.lock().unwrap();
            let agent = exe.new_agent(&rc,name);
            exe.add_pin(Box::pin(f),agent)
        }        
    }

    fn make_lock(&self) -> Lock {
        self.executor.lock().unwrap().make_lock()
    }
}
