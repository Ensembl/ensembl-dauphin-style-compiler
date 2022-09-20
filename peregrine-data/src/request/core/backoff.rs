use commander::cdr_timer;
use crate::{util::message::DataMessage, Channel, PacketPriority};

use super::{manager::RequestManager, request::BackendRequest, response::BackendResponse};

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

    pub(crate) async fn backoff<F,T>(&mut self, req: &BackendRequest, cb: F) -> Result<T,DataMessage>
                                                    where F: Fn(BackendResponse) -> Result<T,String> {
        let channel = self.channel.clone();
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute(channel.clone(),self.priority.clone(),req).await?;
            match cb(resp) {
                Ok(r) => { return Ok(r); },
                Err(e) => { last_error = Some(e); }
            }
            self.manager.message(DataMessage::TemporaryBackendFailure(channel.clone()));
            cdr_timer(500.).await; // XXX configurable
        }
        self.manager.message(DataMessage::FatalBackendFailure(channel.clone()));
        Err(match last_error {
            Some(e) => {
                let e = DataMessage::BackendRefused(channel.clone(),e.to_string());
                self.manager.message(e.clone());
                DataMessage::BackendRefused(channel.clone(),e.to_string())
            },
            None => DataMessage::CodeInvariantFailed("unexpected downcast error in backoff".to_string())
        })
    }
}