use crate::lock;
use std::sync::MutexGuard;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use commander::{ Executor, RunSlot, Lock, RunConfig, TaskHandle, cdr_in_agent, cdr_add, cdr_new_agent, TaskResult, KillReason };
use crate::util::message::DataMessage;

pub fn add_task<R>(commander: &PgCommander, t: PgCommanderTaskSpec<R>) -> TaskHandle<Result<R,DataMessage>> {
    let rc = RunConfig::new(t.slot,t.prio,t.timeout);
    if cdr_in_agent() {
        let agent = cdr_new_agent(Some(rc),&t.name);
         cdr_add(Box::pin(t.task),agent)
    } else {
        let mut commander = commander.0.lock().unwrap();
        let mut exe = commander.executor();
        let agent = exe.new_agent(&rc,&t.name);
        exe.add_pin(Box::pin(t.task),agent)
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

pub trait Commander {
    fn start(&self);
    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=Result<(),DataMessage>> + 'static>>) -> TaskHandle<Result<(),DataMessage>>;
    fn make_lock(&self) -> Lock;
    fn identity(&self) -> u64;
    fn executor(&self) -> MutexGuard<Executor>;
}

pub struct PgCommanderTaskSpec<T> {
    pub name: String,
    pub prio: i8, 
    pub slot: Option<RunSlot>, 
    pub timeout: Option<f64>,
    pub task: Pin<Box<dyn Future<Output=Result<T,DataMessage>>>>
}

#[derive(Clone)]
pub struct PgCommander(Arc<Mutex<Box<dyn Commander>>>);

impl PgCommander {
    pub fn new(c: Box<dyn Commander>) -> PgCommander {
        PgCommander(Arc::new(Mutex::new(c)))
    }

    pub fn dd_task(&self, t: PgCommanderTaskSpec<()>) -> TaskHandle<Result<(),DataMessage>> {
        lock!(self.0).add_task(&t.name,t.prio,t.slot,t.timeout,t.task)
    }
}