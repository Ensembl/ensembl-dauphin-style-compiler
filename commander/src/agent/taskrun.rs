use std::cell::RefCell;
use std::future::Future;
use crate::task::runconfig::RunConfig;
use crate::task::task::KillReason;
use crate::task::taskhandle::TaskHandle;
use super::agent::Agent;

thread_local! {
    static AGENT: RefCell<Option<Agent>> = RefCell::new(None);
}

pub fn cdr_set_agent(agent: Option<&Agent>) {
    AGENT.with(|a| { *a.borrow_mut() = agent.cloned() });
}

pub fn cdr_in_agent() -> bool {
    AGENT.with(|a| a.borrow().is_some())
}

pub fn cdr_get_name() -> String {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().get_name() })
}

pub fn cdr_identity() -> Option<u64> {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().identity() })
}

pub fn cdr_set_name(name: &str) {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().set_name(name) });
}

pub fn cdr_add_timer<T>(timeout: f64, callback: T) where T: FnOnce() + 'static {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().add_timer(timeout,callback) });
}

pub fn cdr_add_ticks_timer<T>(ticks: u64, callback: T) where T: FnOnce() + 'static {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().add_ticks_timer(ticks,callback) });
}

pub fn cdr_new_agent(rc: Option<RunConfig>, name: &str) -> Agent {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().new_agent(rc,name) })
}

pub fn cdr_add<R,T>(future: T, agent: Agent) -> TaskHandle<R> where T: Future<Output=R> + 'static, R: 'static {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().add(future,agent) })
}

pub fn cdr_finish(reason: KillReason) {
    AGENT.with(|a| { a.borrow_mut().as_mut().unwrap().finish(reason) })
}

pub fn cdr_get_config() -> RunConfig {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().get_config() })
}

pub fn cdr_get_tick_index() -> u64 {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().get_tick_index() })
}

pub fn cdr_tick(ticks: u64) -> impl Future<Output=()> {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().tick(ticks) })
}

pub fn cdr_timer(timeout: f64) -> impl Future<Output=()> {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().timer(timeout) })
}

pub fn cdr_current_time() -> f64 {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().get_current_time() })
}

pub fn cdr_turnstile<R,T>(inner: T) -> impl Future<Output=R> where T: Future<Output=R> + 'static {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().turnstile(inner) })
}

pub fn cdr_named_wait<R,T>(inner: T, name: &str) -> impl Future<Output=R> where T: Future<Output=R> + 'static {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().named_wait(inner,name) })
}

pub fn cdr_tidy<T>(inner: T) -> impl Future<Output=()> where T: Future<Output=()> + 'static  {
    AGENT.with(|a| { a.borrow().as_ref().unwrap().tidy(inner) })
}
