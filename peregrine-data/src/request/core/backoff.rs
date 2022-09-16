use commander::cdr_timer;
use crate::{core::channel::{Channel, PacketPriority}, util::message::DataMessage};

use super::{manager::NetworkRequestManager, request::BackendRequest, response::BackendResponse};

pub struct Backoff { 
    manager: NetworkRequestManager,
    channel: Channel,
    priority: PacketPriority
}

impl Backoff {
    pub fn new(manager: &NetworkRequestManager, channel: &Channel, priority: &PacketPriority) -> Backoff {
        Backoff {
            manager: manager.clone(),
            channel: channel.clone(),
            priority: priority.clone()
        }
    }

    pub async fn backoff<F,T>(&mut self, req: BackendRequest, cb: F) -> Result<T,DataMessage>
                                                    where F: Fn(BackendResponse) -> Result<T,String> {
        let channel = self.channel.clone();
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute(channel.clone(),self.priority.clone(),req.clone()).await?;
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