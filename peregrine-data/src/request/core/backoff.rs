use commander::cdr_timer;
use crate::{util::message::DataMessage };

use super::{manager::{LowLevelRequestManager}, request::MiniRequest, response::BackendResponse, queue::QueueKey};

pub struct Backoff { 
    manager: LowLevelRequestManager,
    key: QueueKey
}

impl Backoff {
    pub(crate) fn new(manager: &LowLevelRequestManager, key: &QueueKey) -> Backoff {
        Backoff {
            manager: manager.clone(),
            key: key.clone()
        }
    }

    fn errname(&self) -> String {
        self.key.name.clone().map(|x| x.to_string()).unwrap_or_else(|| "*anon*".to_string())
    }

    pub(crate) async fn backoff<F,T>(&mut self, req: &MiniRequest, cb: F) -> Result<T,DataMessage>
                                                    where F: Fn(BackendResponse) -> Result<T,String> {
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute(&self.key,req)?.get().await;
            match cb(resp) {
                Ok(r) => { return Ok(r); },
                Err(e) => { last_error = Some(e); }
            }
            self.manager.message(DataMessage::TemporaryBackendFailure(self.errname()));
            cdr_timer(500.).await; // XXX configurable
        }
        self.manager.message(DataMessage::FatalBackendFailure(self.errname()));
        Err(match last_error {
            Some(e) => {
                let e = DataMessage::BackendRefused(self.errname(),e.to_string());
                self.manager.message(e.clone());
                DataMessage::BackendRefused(self.errname(),e.to_string())
            },
            None => DataMessage::CodeInvariantFailed("unexpected downcast error in backoff".to_string())
        })
    }
}