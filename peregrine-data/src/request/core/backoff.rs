use std::rc::Rc;
use commander::cdr_timer;
use peregrine_toolkit::error::Error;
use super::{manager::{LowLevelRequestManager}, request::MiniRequest, queue::QueueKey, response::MiniResponseAttempt};

pub struct Backoff { 
    manager: LowLevelRequestManager,
    key: QueueKey,
    repeats: usize,
    pace: bool
}

impl Backoff {
    pub(crate) fn new(manager: &LowLevelRequestManager, key: &QueueKey, enable: bool) -> Backoff {
        Backoff {
            manager: manager.clone(),
            key: key.clone(),
            repeats: if enable { 5 } else { 1 },  // XXX configurable
            pace: enable
        }
    }

    fn errname(&self) -> String {
        self.key.name.clone().map(|x| x.to_string()).unwrap_or_else(|| "*anon*".to_string())
    }

    pub(crate) async fn backoff<F,T>(&mut self, req: &Rc<MiniRequest>, cb: F) -> Result<T,Error>
                                                    where F: Fn(MiniResponseAttempt) -> Result<T,String> {
        let mut last_error = None;
        for _ in 0..self.repeats {
            let resp = self.manager.execute(&self.key,self.pace,req)?.get().await;
            match cb(resp) {
                Ok(r) => { return Ok(r); },
                Err(e) => { last_error = Some(e); }
            }
            self.manager.message(Error::tmp(&format!("temporary backend failure: {}",self.errname())));
            cdr_timer(500.).await; // XXX configurable
        }
        self.manager.message(Error::operr(&format!("permanent backend failure: {}",self.errname())));
        Err(match last_error {
            Some(e) => {
                let e = Error::operr(&format!("backend {} refused: {}",self.errname(),e));
                self.manager.message(e.clone());
                e
            },
            None => Error::fatal("unexpected downcast error in backoff")
        })
    }
}