use anyhow::Context;
use std::pin::Pin;
use std::future::Future;
use std::sync::{ Arc, Mutex };
use commander::{ Integration, SleepQuantity, Executor, RunSlot, RunConfig, cdr_get_name, TaskHandle, TaskResult, cdr_in_agent, cdr_new_agent, cdr_add };
use crate::Commander;
use super::console::TestConsole;

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
    console: TestConsole,
    executor: Arc<Mutex<Executor>>
}

impl TestCommander {
    pub(crate) fn new(console: &TestConsole) -> TestCommander {
        let integration = TestCommanderIntegration::new();
        TestCommander {
            integration: integration.clone(),
            executor: Arc::new(Mutex::new(Executor::new(integration))),
            console: console.clone()
        }
    }

    pub(crate) fn add_time(&self, time: f64) {
        self.integration.add_time(time);
    }

    pub(crate) fn tick(&self) {
        print!("tick\n");
        self.executor.lock().unwrap().tick(1.);
    }
}

pub(crate) fn console_warn(console: &TestConsole, e: anyhow::Result<()>) {
    match e {
        Ok(e) => e,
        Err(e) => {
            console.message(&format!("{:?}",e));
        }
    }
}

async fn catch_errors(console: TestConsole, f: Pin<Box<dyn Future<Output=anyhow::Result<()>>>>) {
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

impl Commander for TestCommander {
    fn start(&self) {
    }

    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>) {
        let rc = RunConfig::new(slot,prio,timeout);
        let rc2 = RunConfig::new(None,prio,None);
        let console = self.console.clone();
        let console2 = self.console.clone();
        if cdr_in_agent() {
            let agent = cdr_new_agent(Some(rc),name);
            let res = cdr_add(Box::pin(catch_errors(console,f)),agent);
            let agent = cdr_new_agent(Some(rc2),&format!("{}-finisher",name));
            cdr_add(finish(console2,res,name.to_string()),agent);
        } else {
            let mut exe = self.executor.lock().unwrap();
            let agent = exe.new_agent(&rc,name);
            let res = exe.add_pin(Box::pin(catch_errors(console,f)),agent);
            let agent = exe.new_agent(&rc2,&format!("{}-finisher",name));
            exe.add(finish(console2,res,name.to_string()),agent);
    
        }        
    }
}
