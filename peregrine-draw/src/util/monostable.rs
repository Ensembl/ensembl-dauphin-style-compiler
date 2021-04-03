use std::sync::{ Arc, Mutex, Weak };
use commander::{ cdr_timer };
use crate::integration::pgcommander::PgCommanderWeb;
use crate::util::message::Message;

struct MonostableState {
    future: bool,
    current: bool
}

async fn monostable_agent(weak_state: Weak<Mutex<MonostableState>>, period_ms: f64, cb: Arc<Box<dyn Fn() + 'static>>) -> Result<(),Message> {
    loop {
        if let Some(state_lock) = weak_state.upgrade() {
            let mut state = state_lock.lock().unwrap();
            if state.current && !state.future {
                cb();
            }
            state.current = state.future;
            state.future = false;
            drop(state);
        } else {
            break;
        }
        cdr_timer(period_ms).await;
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct Monostable(Arc<Mutex<MonostableState>>);

impl Monostable {
    pub(crate) fn new<F>(commander: &PgCommanderWeb, period_ms: f64, cb: F) -> Monostable where F: Fn() + 'static {
        let state = Arc::new(Mutex::new(MonostableState {
            future: false,
            current: false
        }));
        let weak_state = Arc::downgrade(&state);
        let callback : Arc<Box<dyn Fn() + 'static>> = Arc::new(Box::new(cb));
        commander.add("monostable",15,None,None,Box::pin(monostable_agent(weak_state,period_ms,callback)));
        Monostable(state)
    }

    pub(crate) fn set(&self) {
        let mut state = self.0.lock().unwrap();
        state.future = true;
        state.current = true;
    }
    
    pub(crate) fn get(&self) -> bool { self.0.lock().unwrap().current }
}
