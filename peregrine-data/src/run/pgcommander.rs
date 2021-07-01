use std::sync::MutexGuard;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use commander::{ Agent, Executor, RunSlot, Lock, RunConfig, TaskHandle, cdr_in_agent, cdr_add, cdr_new_agent, TaskResult, KillReason };
use crate::api::MessageSender;
use crate::util::message::DataMessage;
use web_sys::console;

async fn then_print_stats<T>(agent: Agent, f: Pin<Box<dyn Future<Output=T>>>) -> T  {
    let out = f.await;
    if agent.stats_enabled() {
        // XXX not web_sys
        //console::log_1(&format!("task timings took={}ms clock={}ms",agent.run_time(),agent.clock_time()).into());
    }
    out
}

pub fn add_task<R>(commander: &PgCommander, t: PgCommanderTaskSpec<R>) -> TaskHandle<Result<R,DataMessage>> {
    let rc = RunConfig::new(t.slot,t.prio,t.timeout);
    let mut task = t.task;
    if cdr_in_agent() {
        let agent = cdr_new_agent(Some(rc),&t.name);
        if t.stats { agent.enable_stats(); task = Box::pin(then_print_stats(agent.clone(),task)); }
         cdr_add(Box::pin(task),agent)
    } else {
        let commander = commander.0.lock().unwrap();
        let mut exe = commander.executor();
        let agent = exe.new_agent(&rc,&t.name);
        if t.stats { agent.enable_stats(); task = Box::pin(then_print_stats(agent.clone(),task)); }
        exe.add_pin(Box::pin(task),agent)
    }
}

pub async fn complete_task<R>(handle: TaskHandle<Result<R,DataMessage>>) -> Result<R,DataMessage> {
    handle.finish_future().await;
    match handle.task_state() {
        TaskResult::Killed(reason) => {
            match reason {
                KillReason::Timeout => { Err(DataMessage::TaskTimedOut(handle.get_name())) },
                KillReason::Cancelled => { Err(DataMessage::TaskUnexpectedlyCancelled(handle.get_name())) },
                KillReason::NotNeeded => { Err(DataMessage::TaskUnexpectedlySuperfluous(handle.get_name())) },
            }
        },
        TaskResult::Done => {
            if let Some(result) = handle.take_result() {
                result
            } else {
                Err(DataMessage::TaskResultMissing(handle.get_name()))
            }
        },
        TaskResult::Ongoing => {
            Err(DataMessage::TaskUnexpectedlyOngoing(handle.get_name()))
        },
    }
}

pub fn async_complete_task<F>(commander: &PgCommander, messages: &MessageSender, handle: TaskHandle<Result<(),DataMessage>>, error: F) 
                                        where F: FnOnce(DataMessage) -> (DataMessage,bool) + 'static {
    let messages = messages.clone();
    add_task(commander,PgCommanderTaskSpec {
        name: format!("catcher"),
        prio: 8,
        slot: None,
        timeout: None,
        task: Box::pin(async move {
            if let Err(r) = complete_task(handle).await {
                let orig_r = r.clone();
                let r = error(r);
                if r.1 {
                    messages.send(orig_r);
                }
                messages.send(r.0);
            }
            Ok(())
        }),
        stats: false
    });
}

pub trait Commander {
    fn start(&self);
    fn add_task(&self, name: &str, prio: u8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=Result<(),DataMessage>> + 'static>>) -> TaskHandle<Result<(),DataMessage>>;
    fn make_lock(&self) -> Lock;
    fn identity(&self) -> u64;
    fn executor(&self) -> MutexGuard<Executor>;
}

pub struct PgCommanderTaskSpec<T> {
    pub name: String,
    pub prio: u8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub task: Pin<Box<dyn Future<Output=Result<T,DataMessage>>>>,
    pub stats: bool
}

#[derive(Clone)]
pub struct PgCommander(Arc<Mutex<Box<dyn Commander>>>);

impl PgCommander {
    pub fn new(c: Box<dyn Commander>) -> PgCommander {
        PgCommander(Arc::new(Mutex::new(c)))
    }
}