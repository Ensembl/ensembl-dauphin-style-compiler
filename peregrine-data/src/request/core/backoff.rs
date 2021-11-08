use commander::cdr_timer;
use crate::{core::channel::{Channel, PacketPriority}, util::message::DataMessage};

use super::{manager::RequestManager, request::RequestType, response::NewResponse};

pub struct Backoff { 
    manager: RequestManager,
    channel: Channel,
    priority: PacketPriority
}

impl Backoff {
    pub fn new(manager: &RequestManager, channel: &Channel, priority: &PacketPriority) -> Backoff {
        Backoff {
            manager: manager.clone(),
            channel: channel.clone(),
            priority: priority.clone()
        }
    }

    pub async fn backoff<F,T>(&mut self, req: RequestType, cb: F) -> Result<T,DataMessage>
                                                    where F: Fn(NewResponse) -> Result<T,String> {
        let channel = self.channel.clone();
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute_new(channel.clone(),self.priority.clone(),req.clone()).await?;
            match cb(resp) {
                Ok(r) => { return Ok(r); },
                Err(e) => { last_error = Some(e); }
            }
            self.manager.message(DataMessage::TemporaryBackendFailure(channel.clone()));
            cdr_timer(500.).await; // XXX configurable
        }
        self.manager.message(DataMessage::FatalBackendFailure(channel.clone()));
        Err(match last_error {
            Some(e) => DataMessage::BackendRefused(channel.clone(),e.to_string()),
            None => DataMessage::CodeInvariantFailed("unexpected downcast error in backoff".to_string())
        })
    }
}