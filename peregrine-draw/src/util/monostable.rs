use std::sync::{ Arc, Mutex, Weak };
use commander::{ cdr_timer };
use peregrine_toolkit_async::sync::needed::{Needed, NeededLock};
use crate::integration::pgcommander::PgCommanderWeb;
use crate::util::message::Message;
struct MonostableState {
    needed: Needed,
    future: bool,
    current: bool,
    #[allow(unused)]
    lock: Option<NeededLock>
}

async fn monostable_agent(weak_state: Weak<Mutex<MonostableState>>, needed: Needed, period_ms: f64, cb: Arc<Box<dyn Fn() + 'static>>) -> Result<(),Message> {
    loop {
        if let Some(state_lock) = weak_state.upgrade() {
            let mut state = state_lock.lock().unwrap();
            if state.current && !state.future {
                cb();
            }
            state.current = state.future;
            state.future = false;
            if !state.future && !state.current { state.lock = None; }
            drop(state);
        } else {
            break;
        }
        cdr_timer(period_ms).await;
        needed.wait_until_needed().await;
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct Monostable(Arc<Mutex<MonostableState>>);

// XXX run level audit
impl Monostable {
    pub(crate) fn new<F>(commander: &PgCommanderWeb, period_ms: f64, cb: F) -> Monostable where F: Fn() + 'static {
        let needed = Needed::new();
        let state = Arc::new(Mutex::new(MonostableState {
            lock: None,
            future: false,
            current: false,
            needed: needed.clone()
        }));
        let weak_state = Arc::downgrade(&state);
        let callback : Arc<Box<dyn Fn() + 'static>> = Arc::new(Box::new(cb));
        commander.add("monostable",6,None,None,Box::pin(monostable_agent(weak_state,needed,period_ms,callback)));
        Monostable(state)
    }

    pub(crate) fn set(&self) {
        let mut state = self.0.lock().unwrap();
        state.future = true;
        state.current = true;
        state.lock = Some(state.needed.lock());
    }
    
    pub(crate) fn get(&self) -> bool { 
        self.0.lock().unwrap().current
    }
}
