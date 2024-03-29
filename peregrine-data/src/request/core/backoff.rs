use std::rc::Rc;
use commander::cdr_timer;
use peregrine_toolkit::error::Error;
use super::{manager::{LowLevelRequestManager}, minirequest::MiniRequest, queue::QueueKey, miniresponse::{MiniResponseAttempt, MiniResponseError}};

pub struct Backoff { 
    manager: LowLevelRequestManager,
    key: QueueKey,
    repeats: usize
}

impl Backoff {
    pub(crate) fn new(manager: &LowLevelRequestManager, key: &QueueKey, repeats: usize) -> Backoff {
        Backoff {
            manager: manager.clone(),
            key: key.clone(),
            repeats
        }
    }

    fn errname(&self) -> String {
        self.key.name.clone().map(|x| x.to_string()).unwrap_or_else(|| "*anon*".to_string())
    }

    pub(crate) async fn backoff<F,T>(&mut self, req: &Rc<MiniRequest>, cb: F) -> Result<T,Error>
            where F: Fn(MiniResponseAttempt) -> Result<T,MiniResponseError> {
        let mut last_error = None;
        for _ in 0..self.repeats {
            let resp = self.manager.execute(&self.key,req)?.get().await;
            match cb(resp) {
                Ok(r) => { return Ok(r); },
                Err(MiniResponseError::Retry(e)) => { last_error = Some(e); },
                Err(MiniResponseError::NoRetry(e)) => { last_error = Some(e); break; },
            }
            self.manager.message(Error::tmp(&format!("temporary backend failure: {}",self.errname())));
            cdr_timer(500.).await;
        }
        self.manager.message(Error::operr(&format!("permanent backend failure: {}",self.errname())));
        Err(last_error.unwrap_or_else(|| Error::fatal("unexpected downcast error in backoff")))
    }
}
