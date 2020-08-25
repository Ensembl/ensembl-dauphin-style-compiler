use anyhow::{ self, Context, anyhow as err };
use blackbox::blackbox_log;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use std::future::Future;
use owning_ref::MutexGuardRefMut;
use commander::{ cdr_get_name, Executor, Integration, RunConfig, RunSlot, SleepQuantity, TaskHandle, TaskResult, cdr_new_agent, cdr_add, cdr_in_agent };
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::Date;
use crate::util::error::{ js_error, js_option, js_throw, console_error, js_warn };
use super::bell::{ BellReceiver, make_bell, BellSender };
use web_sys::{ HtmlElement };
use peregrine_core::Commander;

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

struct CommanderSleepState {
    raf_pending: Option<Closure<dyn Fn()>>,
    quantity: Arc<Mutex<SleepQuantity>>,
    timeout: Option<(Closure<dyn Fn()>,i32)>
}

#[derive(Clone)]
struct CommanderState {
    sleep_state: Arc<Mutex<CommanderSleepState>>,
    executor: Arc<Mutex<Executor>>
}

impl CommanderState {
    fn tick(&self) {
        self.executor.lock().unwrap().tick(MS_PER_TICK);
    }

    fn raf_tick(&self) {
        self.sleep_state.lock().unwrap().raf_pending = None;
        self.tick();
        js_throw(self.schedule());
    }

    fn timer_tick(&self) {
        self.sleep_state.lock().unwrap().timeout.take();
        self.tick();
        js_throw(self.schedule());
    }

    fn schedule(&self) -> anyhow::Result<()> {
        let window = js_option(web_sys::window(),"cannot get window")?;
        let mut state = self.sleep_state.lock().unwrap();
        if let Some((_,handle)) = state.timeout.take() {
            window.clear_timeout_with_handle(handle);
        }
        let quantity = state.quantity.lock().unwrap().clone();
        match quantity {
            SleepQuantity::Forever => {},
            SleepQuantity::None => {
                let handle = self.clone();
                if state.raf_pending.is_none() {
                    state.raf_pending = Some(Closure::wrap(Box::new(move || {
                        handle.raf_tick()
                    })));
                    js_error(window.request_animation_frame(state.raf_pending.as_ref().unwrap().as_ref().unchecked_ref()))?;
                }
            },
            SleepQuantity::Time(t) => {
                let handle = self.clone();
                let closure = Closure::wrap(Box::new(move || {
                    handle.timer_tick()
                }) as Box<dyn Fn()>);
                let handle = js_error(window.set_timeout_with_callback_and_timeout_and_arguments_0(closure.as_ref().unchecked_ref(),t as i32))?;
                state.timeout = Some((closure,handle));
            }
        }
        Ok(())
    }
}

async fn catch_errors(f: Pin<Box<dyn Future<Output=anyhow::Result<()>>>>) {
    js_warn(f.await.with_context(|| format!("async: {}",cdr_get_name())));
}

async fn finish(res: TaskHandle<()>, name: String) {
    res.finish_future().await;
    match res.task_state() {
        TaskResult::Killed(reason) => {
            console_error(&format!("async {}: {}",reason,name));
        },
        _ => {}
    }
}

#[derive(Clone)]
pub struct PgCommanderWeb {
    state: CommanderState,
    bell_receiver: BellReceiver
}

impl PgCommanderWeb {
    pub fn new(el: &HtmlElement) -> anyhow::Result<PgCommanderWeb> {
        let quantity = Arc::new(Mutex::new(SleepQuantity::Forever));
        let sleep_state = Arc::new(Mutex::new(CommanderSleepState {
            raf_pending: None,
            quantity: quantity.clone(),
            timeout: None
        }));
        let (bell_sender, bell_receiver) = make_bell(el)?;
        let integration = PgIntegration {
            quantity,
            bell_sender
        };
        let state = CommanderState {
            sleep_state,
            executor: Arc::new(Mutex::new(Executor::new(integration)))
        };
        let mut out = PgCommanderWeb {
            state: state.clone(),
            bell_receiver
        };
        out.bell_receiver.add(move || {
            blackbox_log!("commander-integration","bell received");
            js_throw(state.schedule());
        });
        Ok(out)
    }
}

impl Commander for PgCommanderWeb {
    fn start(&self) {
        self.state.tick();
    }

    fn add_task(&self, name: &str, prio: i8, slot: Option<RunSlot>, timeout: Option<f64>, f: Pin<Box<dyn Future<Output=anyhow::Result<()>> + 'static>>) {
        let rc = RunConfig::new(slot,prio,timeout);
        let rc2 = RunConfig::new(None,prio,None);
        if cdr_in_agent() {
            let agent = cdr_new_agent(Some(rc),name);
            let res = cdr_add(Box::pin(catch_errors(f)),agent);
            let agent = cdr_new_agent(Some(rc2),&format!("{}-finisher",name));
            cdr_add(finish(res,name.to_string()),agent);
        } else {
            let mut exe = self.state.executor.lock().unwrap();
            let agent = exe.new_agent(&rc,name);
            let res = exe.add_pin(Box::pin(catch_errors(f)),agent);
            let agent = exe.new_agent(&rc2,&format!("{}-finisher",name));
            exe.add(finish(res,name.to_string()),agent);
        }        
    }
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
        blackbox_log!("commander-integration","setting sleep to {:?}",amount);
        *self.quantity.lock().unwrap() = amount;
        js_throw(self.bell_sender.ring());
        blackbox_log!("commander-integration","bell sent");
    }
}