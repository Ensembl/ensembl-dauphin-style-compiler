use futures::task::{ Context, waker_ref };
use owning_ref::MutexGuardRefMut;
use peregrine_toolkit::{sample_str };
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use std::task::Poll;
use crate::executor::action::Action;
use crate::executor::link::Link;
use crate::executor::lock::{ Lock, LockGuard };
use crate::executor::request::Request;
use crate::corefutures::promisefuture::PromiseFuture;
use crate::corefutures::namedfuture::NamedFuture;
use crate::corefutures::turnstilefuture::TurnstileFuture;
use crate::integration::reentering::ReenteringIntegration;
use crate::task::runconfig::RunConfig;
use crate::task::task::KillReason;
use crate::task::taskhandle::TaskHandle;
use super::blockagent::BlockAgent;
use super::finishagent::FinishAgent;
use super::nameagent::NameAgent;
use super::runagent::RunAgent;
use super::taskrun::cdr_set_agent;

#[cfg(debug_timehogs)]
use peregrine_toolkit::{ log, time::now };

/* An agent provides methods on behalf of an Executor for use within a future. It
 * is also ultimately responsible for executing the future and for destructors.
 * That's a lot of work, so it's split into mixins. In this main class there are:
 *   1. accessors for the mixins (crate scoped);
 *   2. public API;
 *   3. top-level execution methods (as they require coordination across mixins)
 */

pub(crate) struct AgentState {
    block_agent: BlockAgent,
    finish_agent: FinishAgent,
    name_agent: NameAgent,
    run_agent: RunAgent
}

#[derive(Clone)]

/// Agents provide access to executor functionality from within a running future.
/// 
/// This allows them to create and wait on timers, create subtasks, etc. For an overview see the top-level 
/// documentation.
pub struct Agent {
    state: Arc<Mutex<AgentState>>
}

impl Agent {
    pub(crate) fn new(config: &RunConfig,
                      action_link: &Link<Action>, request_link: &Link<Request>,
                      integration: &ReenteringIntegration, name: &str) -> Agent {
        let task_action_link = action_link.new_task_link();
        let task_request_link = request_link.new_task_link();
        let out = Agent {
            state: Arc::new(Mutex::new(AgentState {
                block_agent: BlockAgent::new(integration,&task_action_link),
                finish_agent: FinishAgent::new(integration,&task_action_link),
                name_agent: NameAgent::new(name),
                run_agent: RunAgent::new(integration,&task_action_link,&task_request_link,config)
            }))
        };
        out
    }

    /* access mixins */

    pub(crate) fn block_agent(&self) -> MutexGuardRefMut<AgentState,BlockAgent> {
        MutexGuardRefMut::new(self.state.lock().unwrap()).map_mut(|x| &mut x.block_agent)
    }

    pub(crate) fn finish_agent(&self) -> MutexGuardRefMut<AgentState,FinishAgent> {
        MutexGuardRefMut::new(self.state.lock().unwrap()).map_mut(|x| &mut x.finish_agent)
    }

    pub(crate) fn name_agent(&self) -> MutexGuardRefMut<AgentState,NameAgent> {
        MutexGuardRefMut::new(self.state.lock().unwrap()).map_mut(|x| &mut x.name_agent)
    }

    pub(crate) fn run_agent(&self) -> MutexGuardRefMut<AgentState,RunAgent> {
        MutexGuardRefMut::new(self.state.lock().unwrap()).map_mut(|x| &mut x.run_agent)
    }

    /* public API */

    /// Retrieve current name of task.
    pub fn get_name(&self) -> String {
        self.name_agent().get_name()
    }

    /// Set name of task.
    pub fn set_name(&self, name: &str) {
        self.name_agent().set_name(name);
    }

    /// Get unique numeric identity for running task.
    pub fn identity(&self) -> Option<(u64,u64)> {
        self.run_agent().get_id()
    }

    /// Add a callback to be run by the executor in a tick after the given timeout.
    /// 
    /// Lower-level than `timer()` and generally less useful an interface to the same functionality.
    pub fn add_timer<T>(&self, timeout: f64, callback: T) where T: FnOnce() + 'static {
        self.run_agent().add_timer(timeout,callback);
    }

    /// Convenience method to get current time as supplied by integration.
    pub(super) fn get_current_time(&self) -> f64 {
        self.run_agent().get_current_time()
    }

    /// Add a callback to be run by the executor after the given number of ticks. zero implies a yield.
    /// 
    /// Lower-level than `tick()` and not generally as useful an interface to the same functionality.
    pub fn add_ticks_timer<T>(&self, ticks: u64, callback: T) where T: FnOnce() + 'static {
        self.run_agent().add_ticks_timer(ticks,callback);
    }

    /// Create a new agent for eventual submission of a subtask with `submit()`.
    /// 
    /// If rc is `None`, the rc of this task is reused. As with top-level submission direct to the executor, keeping 
    /// `new_agent()` and `submit()` separate allows the agent to be cloned and moved into the new future.
    pub fn new_agent(&self, rc: Option<RunConfig>, name: &str) -> Agent {
        self.run_agent().new_agent(name,rc)
    }

    /// Submit a given agent and future for running as a new, independent task.
    /// 
    /// The agent passed should have been created with `new_agent()`. Do not pass self!
    pub fn add<R,T>(&self, future: T, agent: Agent) -> TaskHandle<R> where T: Future<Output=R> + 'static, R: 'static {
        self.run_agent().submit(agent,future)
    }

    /// Cause the current task to finish via signal with the given reason.
    pub fn finish(&self, reason: KillReason) {
        self.finish_agent().finish(Some(&reason),true);
    }

    /// Get RunConfig used by this task
    pub fn get_config(&self) -> RunConfig { self.run_agent().get_config().clone() }

    /// Get the current tick number.
    /// 
    /// The tick number advances by one each tick.
    pub fn get_tick_index(&self) -> u64 { self.run_agent().get_tick_index() }

    /// Return a `Future` which waits the given number of ticks.
    /// 
    /// It is a good idea to call this with value `0` and await if you are inside a tight loop in your future. This
    /// allows the tick timeout to be checked and for other tasks of the same priority to run. If there is still "time
    /// on the clock" after that, your future will resume, even in the same tick.
    pub fn tick(&self,ticks: u64) -> impl Future<Output=()> {
        let future = PromiseFuture::new();
        let future2 = future.clone();
        self.add_ticks_timer(ticks, move || future2.satisfy(()));
        future
    }

    /// Return a `Future` which waits for the given time.
    /// 
    /// Time units are those of your integration.
    pub fn timer(&self, timeout: f64) -> impl Future<Output=()> {
        let future = PromiseFuture::new();
        let future2 = future.clone();
        self.add_timer(timeout, move || future2.satisfy(()));
        future
    }

    /// Return turnstile `Future` for more efficient waits.
    /// 
    /// Typically in a Rust futures executor, when some wake-up occurs within a futures tree, the whole task is 
    /// rescanned, including other branches which may be blocked and not awoken. This can potentially be problematic for
    /// very large tasks where rechecking is expensive. 
    /// 
    /// For further details see the crate overview.
    pub fn turnstile<R,T>(&self, inner: T) -> impl Future<Output=R> where T: Future<Output=R> + 'static {
        TurnstileFuture::new(&self,inner)
    }

    /// Return a named wait `Future` for diagnostic purposes.
    /// 
    /// When the future wrapped in this named_wait is waiting its name is added to a list accessible from this
    /// task's summary. This allows the waiting to be evidenced for diagnosis.
    pub fn named_wait<R,T>(&self, inner: T, name: &str) -> impl Future<Output=R> where T: Future<Output=R> + 'static {
        NamedFuture::new(&self,inner,name)
    }

    /// Return a tidier: a kind of destructor.
    /// 
    /// This future can be waited on directly but, if it isn't, then on completion it is run anyway, even if abandoned
    /// with a signal.
    pub fn tidy<T>(&self, inner: T) -> impl Future<Output=()> where T: Future<Output=()> + 'static  {
        self.finish_agent().make_tidier(inner)
    }

    pub(crate) fn lock(&self, lock: &Lock) -> impl Future<Output=LockGuard<'static>> {
        self.run_agent().lock(lock)
    }

    // Enable statistics collection. (For times)
    pub fn enable_stats(&self) { self.run_agent().enable_stats(); }

    // Return clock time since task start (only valid if stats enabled)
    pub fn clock_time(&self) -> f64 { self.run_agent().clock_time() }
 
    // Return total time spent running (only valid if stats enabled)
    pub fn run_time(&self) -> f64 { self.run_agent().run_time() }

    // Are stats enabled?
    pub fn stats_enabled(&self) -> bool { self.run_agent().state_enabled() }

    /* overall run task */

    fn run_one_destructor(&self, context: &mut Context) {
        self.finish_agent().check_tidiers();
        let mut tidier = self.finish_agent().get_tidier().cloned();
        if let Some(ref mut tidier) = tidier {
            match tidier.as_mut().poll(context) {
                Poll::Pending => {
                    self.block_agent().root_block().block();
                    self.block_agent().block_task();
                },
                Poll::Ready(_) => {}
            }
        }
    }

    #[cfg(debug_timehogs)]
    fn poll<R>(&self, context: &mut Context, future: &mut Pin<Box<dyn Future<Output=R> + 'static>>) -> Poll<R> {
        let start = now();
        let out = future.as_mut().poll(context);
        if now() - start > 10. {
            log!("Time hog {}ms: {}",now()-start,self.name_agent().get_name());
        }
        out
    }

    #[cfg(not(debug_timehogs))]
    fn poll<R>(&self, context: &mut Context, future: &mut Pin<Box<dyn Future<Output=R> + 'static>>) -> Poll<R> {
        future.as_mut().poll(context)
    }

    fn run_one_main<R>(&self, context: &mut Context, future: &mut Pin<Box<dyn Future<Output=R> + 'static>>, result: &mut Option<R>) {
        sample_str!(self.name_agent().get_name());
        let out = self.poll(context,future);
        sample_str!("".to_string());
        match out {
            Poll::Pending => {
                self.block_agent().root_block().block();
                self.block_agent().block_task();
            },
            Poll::Ready(v) => {
                self.finish_agent().finish(None,false);
                *result = Some(v);
            }
        };
    }

    pub(crate) fn more<R>(&self, future: &mut Pin<Box<dyn Future<Output=R> + 'static>>, tick_index: u64, result: &mut Option<R>) -> bool {
        cdr_set_agent(Some(self));
        self.run_agent().set_tick_index(tick_index);
        if self.finish_agent().finished() {
            return true;
        }
        let waker = self.block_agent().root_block().make_waker();
        let wr = &*waker_ref(&waker);
        let context = &mut Context::from_waker(wr);
        let slice_start = self.run_agent().stats_time();
        if self.finish_agent().finishing() {
            self.run_one_destructor(context);
        } else {
            self.run_one_main(context,future,result);
        }
        let slice_end = self.run_agent().stats_time();
        self.run_agent().timing(slice_start,slice_end);
        self.finish_agent().check_tidiers();
        let out = self.finish_agent().finished();
        cdr_set_agent(None);
        out
    }
}
