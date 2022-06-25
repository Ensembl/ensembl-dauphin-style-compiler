use std::sync::{ Arc, Mutex, Weak };
use commander::{ cdr_timer };
use peregrine_toolkit::lock;
use peregrine_toolkit::plumbing::oneshot::OneShot;
use peregrine_toolkit_async::sync::needed::{Needed};
use crate::integration::pgcommander::PgCommanderWeb;
use crate::util::message::Message;

async fn debounce_agent(needed: Weak<Mutex<Needed>>, period_ms: f64, shutdown: OneShot, cb: Arc<Box<dyn Fn() + 'static>>) -> Result<(),Message> {
    loop {
        if let Some(needed) = needed.upgrade() {
            let needed = lock!(needed).clone();
            needed.wait_until_needed().await;
            cb();
        } else {
            break;
        }
        cdr_timer(period_ms).await;
        if shutdown.poll() { break; }
    }
    Ok(())
}

#[derive(Clone)]
pub(crate) struct Debounce(Arc<Mutex<Needed>>);

// XXX run level audit
impl Debounce {
    pub(crate) fn new<F>(commander: &PgCommanderWeb, period_ms: f64, shutdown: &OneShot, cb: F) -> Debounce where F: Fn() + 'static {
        let needed = Arc::new(Mutex::new(Needed::new()));
        let weak_needed = Arc::downgrade(&needed);
        let callback : Arc<Box<dyn Fn() + 'static>> = Arc::new(Box::new(cb));
        let shutdown = shutdown.clone();
        commander.add("debounce",6,None,None,Box::pin(debounce_agent(weak_needed,period_ms,shutdown,callback)));
        Debounce(needed)
    }

    pub(crate) fn set(&self) {
        self.0.lock().unwrap().set();
    }    
}
