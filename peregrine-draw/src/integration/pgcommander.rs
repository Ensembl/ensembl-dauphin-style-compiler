use std::pin::Pin;
use std::sync::{ Arc, Mutex, MutexGuard, Weak };
use std::future::Future;
use commander::{ Executor, Integration, Lock, RunConfig, RunSlot, SleepQuantity, TaskHandle, cdr_new_agent, cdr_add, cdr_in_agent };
use peregrine_toolkit::lock;
use js_sys::Date;
use super::bell::{ BellReceiver, make_bell, BellSender };
use super::raf::Raf;
use super::timer::Timer;
use peregrine_data::{ Commander, DataMessage };
use crate::util::message::{ message, Message };

/* The entity relationship here is crazy complex. This is all to allow non-Send methods in Executor. The BellReceiver
 * needs to be able to call schedule and so needs a reference to both the sleep state (to check it) and the executor
 * (to build a callback). So, to avoid reference loops we need the Executor and the sleep state to be inside an inner
 * object (CommanderState) and the BellReciever outside it, in the Commander. We need the Executor to be inside an
 * Arc because it isn't Clone but CommanderState (which it is within) must be Clone. We can't put the whole of
 * CommanderState inside an Arc because we need easy access to executor from Commander to the outside world, ie not
 * protedcted by nested Arcs. Anyway, CommanderSleepState (the other entry in CommanderState) needs to be in its own
 * Arc anyway as it's shared with the CommanderIntegration.
 *
 * So the dependency graph is:
 *
 *                                                            Arc
 * Commander -+-> BellReceiver ---> CommanderState (clone 1) --+---> CommanderSleepState
 *            +-------------------> CommanderState|(clone 2) --+
 *                                                ||           +--------------------------------+
 *                                            Arc ++---> Executor -> CommanderIntegration -> BellSender
 *
 * The Mutexes must be locked in the order Excecutor before CommanderSleepState (if both are to be held). This avoids
 * deadlock. CommanderSleepState is not kept locked between public methods and is not exposed. Executor is exposed but
 * this guarantees externally no harm can be done. Internally, except in schedule() CommanderSleepState is held only
 * momentarily to update data fields, so that is safe. Inside schedule, only callbacks can (indirectly) take a lock
 * on executor (via raf_tick or timer_tick) and callbacks don't run until we return to the main loop. In a threaded
 * world, they would block until schedule exited.
 */

const MS_PER_TICK : f64 = 7.;

pub fn js_panic(e: Result<(),Message>) {
    match e {
        Ok(_) => (),
        Err(e) => {
            message(e);
        }
    }
}

struct CommanderSleepState {
    raf: Option<Raf>,
    timer: Option<Timer>,
    yesterday: Option<Timer>,
    quantity: Arc<Mutex<SleepQuantity>>
}

#[derive(Clone)]
struct CommanderState {
    sleep_state: Arc<Mutex<CommanderSleepState>>,
    executor: Arc<Mutex<Executor>>,
}

struct WeakCommanderState {
    sleep_state: Weak<Mutex<CommanderSleepState>>,
    executor: Weak<Mutex<Executor>>,
}

impl WeakCommanderState {
    fn upgrade(&self) -> Option<CommanderState> {
        Some(CommanderState {
            sleep_state: if let Some(x) = Weak::upgrade(&self.sleep_state) { x } else { return None; },
            executor: if let Some(x) = Weak::upgrade(&self.executor) { x } else { return None; },
        })
    }
}

impl CommanderState {
    fn downgrade(&self) -> WeakCommanderState {
        WeakCommanderState {
            sleep_state: Arc::downgrade(&self.sleep_state.clone()),
            executor: Arc::downgrade(&self.executor.clone())
        }
    }

    fn yesterday(&self) {
        let mut state = self.sleep_state.lock().unwrap();
        state.yesterday.as_mut().unwrap().go(0);
    }

    fn tick(&self) {
        self.executor.lock().unwrap().tick(MS_PER_TICK);
    }

    fn cb_tick(&self) {
        self.tick();
        js_panic(self.schedule());
    }

    fn make_lock(&self) -> Lock { self.executor.lock().unwrap().make_lock() }
    fn identity(&self) -> u64 { self.executor.lock().unwrap().identity() }

    fn schedule(&self) -> Result<(),Message> {
        let mut state = self.sleep_state.lock().unwrap();
        let quantity = state.quantity.lock().unwrap().clone();
        match quantity {
            SleepQuantity::Yesterday => {
                drop(state);
                self.yesterday();
            },
            SleepQuantity::Forever => {},
            SleepQuantity::None => {
                state.raf.as_mut().unwrap().go();
            },
            SleepQuantity::Time(t) => {
                state.timer.as_mut().unwrap().go(t as i32);
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgCommanderWeb {
    state: CommanderState,
    bell_receiver: BellReceiver
}

impl PgCommanderWeb {
    pub fn new() -> Result<PgCommanderWeb,Message> {
        let quantity = Arc::new(Mutex::new(SleepQuantity::Forever));
        let sleep_state = Arc::new(Mutex::new(CommanderSleepState {
            quantity: quantity.clone(),
            raf: None,
            yesterday: None,
            timer: None
        }));
        let (bell_sender, bell_receiver) = make_bell()?;
        let integration = PeregrineCommanderIntegration {
            quantity,
            bell_sender
        };
        let state = CommanderState {
            sleep_state,
            executor: Arc::new(Mutex::new(Executor::new(integration)))
        };
        let weak_state = state.downgrade();
        lock!(state.sleep_state).raf = Some(Raf::new( move || {
            if let Some(state) = weak_state.upgrade() {
                state.cb_tick();
            }
        }));
        let weak_state = state.downgrade();
        lock!(state.sleep_state).timer = Some(Timer::new( move || {
            if let Some(state) = weak_state.upgrade() {
                state.cb_tick();
            }
        }));
        let weak_state = state.downgrade();
        lock!(state.sleep_state).yesterday = Some(Timer::new( move || {
            if let Some(state) = weak_state.upgrade() {
                state.cb_tick();
            }
        }));
        let mut out = PgCommanderWeb {
            state: state.clone(),
            bell_receiver
        };
        out.bell_receiver.add(move || {
            js_panic(state.schedule());
        });
        Ok(out)
    }

    pub fn add<E>(&self, name: &str, prio: u8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=Result<(),E>> + 'static>>) -> TaskHandle<Result<(),E>> {
        let rc = RunConfig::new(slot,prio,timeout);
        if cdr_in_agent() {
            let agent = cdr_new_agent(Some(rc),name);
             cdr_add(Box::pin(f),agent)
        } else {
            let mut exe = self.state.executor.lock().unwrap();
            let agent = exe.new_agent(&rc,name);
            exe.add_pin(Box::pin(f),agent)
        }
    }
}

// TODO check all add-tasks check result.
impl Commander for PgCommanderWeb {
    fn start(&self) {
        self.state.tick();
    }

    fn add_task(&self, name: &str, prio: u8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=Result<(),DataMessage>> + 'static>>) -> TaskHandle<Result<(),DataMessage>> {
        self.add(name,prio,slot,timeout,f)
    }

    fn make_lock(&self) -> Lock { self.state.make_lock() }
    fn identity(&self) -> u64 { self.state.identity() }

    fn executor(&self) -> MutexGuard<Executor> { self.state.executor.lock().unwrap() }
}

#[derive(Clone)]
struct PeregrineCommanderIntegration {
    quantity: Arc<Mutex<SleepQuantity>>,
    bell_sender: BellSender
}

impl Integration for PeregrineCommanderIntegration {
    fn current_time(&self) -> f64 {
        Date::now()
    }

    fn sleep(&self, amount: SleepQuantity) {
        *self.quantity.lock().unwrap() = amount;
       self.bell_sender.ring();
    }
}
