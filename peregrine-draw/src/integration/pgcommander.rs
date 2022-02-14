use std::pin::Pin;
use std::sync::{ Arc, Mutex, MutexGuard };
use std::future::Future;
use commander::{ Executor, Integration, Lock, RunConfig, RunSlot, SleepQuantity, TaskHandle, cdr_new_agent, cdr_add, cdr_in_agent };
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Date;
use super::bell::{ BellReceiver, make_bell, BellSender };
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
    raf_pending: bool,
    raf_closure: Option<Closure<dyn Fn()>>,
    timer_closure: Option<Closure<dyn Fn()>>,
    quantity: Arc<Mutex<SleepQuantity>>,
    timeout: Option<i32>
}

#[derive(Clone)]
struct CommanderState {
    sleep_state: Arc<Mutex<CommanderSleepState>>,
    executor: Arc<Mutex<Executor>>,
}

impl CommanderState {
    fn yesterday(&self) {
        let window = web_sys::window().unwrap(); // XXX errors
        let mut state = self.sleep_state.lock().unwrap();
        if let Some(handle) = state.timeout.take() {
            window.clear_timeout_with_handle(handle);
        }
        let js_closure = state.timer_closure.as_ref().unwrap().as_ref();
        let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(js_closure.unchecked_ref(),0).ok(); // XXX errors
        state.timeout = handle;
    }

    fn tick(&self) {
        self.executor.lock().unwrap().tick(MS_PER_TICK);
    }

    fn raf_tick(&self) {
        let mut state = self.sleep_state.lock().unwrap();
        state.raf_pending = false;
        drop(state);
        self.tick();
        js_panic(self.schedule());
    }

    fn timer_tick(&self) {
        let mut state = self.sleep_state.lock().unwrap();
        state.timeout.take();
        drop(state);
        self.tick();
        js_panic(self.schedule());
    }

    fn make_lock(&self) -> Lock { self.executor.lock().unwrap().make_lock() }
    fn identity(&self) -> u64 { self.executor.lock().unwrap().identity() }

    fn schedule(&self) -> Result<(),Message> {
        let window = web_sys::window().ok_or_else(|| Message::ConfusedWebBrowser(format!("cannot get window")))?;
        let mut state = self.sleep_state.lock().unwrap();
        let quantity = state.quantity.lock().unwrap().clone();
        match quantity {
            SleepQuantity::Yesterday => {
                drop(state);
                self.yesterday();
            },
            SleepQuantity::Forever => {},
            SleepQuantity::None => {
                if !state.raf_pending {
                    state.raf_pending = true;
                    window.request_animation_frame(state.raf_closure.as_ref().unwrap().as_ref().unchecked_ref()).map_err(|e| Message::ConfusedWebBrowser(format!("cannot create RAF callback: {:?}",e.as_string())))?;
                }
            },
            SleepQuantity::Time(t) => {
                let js_closure = state.timer_closure.as_ref().unwrap().as_ref();
                let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(js_closure.unchecked_ref(),t as i32).map_err(|e| Message::ConfusedWebBrowser(format!("cannot create timeout B: {:?}",e)))?;
                if let Some(handle) = state.timeout.take() {
                    window.clear_timeout_with_handle(handle);
                }        
                state.timeout = Some(handle);
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
            raf_pending: false,
            quantity: quantity.clone(),
            timeout: None,
            raf_closure: None,
            timer_closure: None,
        }));
        let (bell_sender, bell_receiver) = make_bell()?;
        let integration = PgIntegration {
            quantity,
            bell_sender
        };
        let state = CommanderState {
            sleep_state,
            executor: Arc::new(Mutex::new(Executor::new(integration)))
        };
        let state2 = state.clone();
        state.sleep_state.lock().unwrap().raf_closure = Some(Closure::wrap(Box::new(move || {
            state2.raf_tick()
        })));
        let state2 = state.clone();
        state.sleep_state.lock().unwrap().timer_closure = Some(Closure::wrap(Box::new(move || {
            state2.timer_tick()
        })));
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
pub struct PgIntegration {
    quantity: Arc<Mutex<SleepQuantity>>,
    bell_sender: BellSender
}

impl Integration for PgIntegration {
    fn current_time(&self) -> f64 {
        Date::now()
    }

    fn sleep(&self, amount: SleepQuantity) {
        *self.quantity.lock().unwrap() = amount;
        js_panic(self.bell_sender.ring());
    }
}
