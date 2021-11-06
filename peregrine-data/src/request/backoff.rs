use commander::cdr_timer;
use std::any::Any;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::failure::GeneralFailure;
use super::request::{NewResponse, RequestType };
use crate::util::message::DataMessage;

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

    fn downcast<S: 'static>(&self, response: NewResponse) -> Result<Result<Box<S>,Box<dyn Any>>,DataMessage> {
        match response {
            NewResponse::Jump(_) | NewResponse::GeneralFailure(_) => {
                return Err(DataMessage::PacketError(self.channel.clone(),format!("unexpected response to request: new")));
            },
            NewResponse::Other(resp) => {
                match resp.into_any().downcast::<S>() {
                    Ok(s) => {
                        /* Got expected response */
                        return Ok(Ok(s));
                    },
                    Err(resp) => {
                        match resp.downcast::<GeneralFailure>() {
                            /* Got general failure */
                            Ok(e) => { 
                                self.manager.message(DataMessage::BackendRefused(self.channel.clone(),e.message().to_string()));
                                return Ok(Err(e));
                            },
                            Err(e) => {
                                /* Gor something unexpected */
                                return Err(DataMessage::PacketError(self.channel.clone(),format!("unexpected response to request: {:?}",e)));
                            }
                        }
                    }
                }
            }
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

    pub async fn backoff_new<S>(&mut self, req: RequestType) -> Result<Result<Box<S>,DataMessage>,DataMessage>
                    where S: 'static {
        let channel = self.channel.clone();
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute_new(channel.clone(),self.priority.clone(),req.clone()).await?;
            match self.downcast(resp)? {
                Ok(result) => { return Ok(Ok(result)); },
                Err(s) => { last_error = Some(s); }
            }
            self.manager.message(DataMessage::TemporaryBackendFailure(channel.clone()));
            cdr_timer(500.).await; // XXX configurable
        }
        self.manager.message(DataMessage::FatalBackendFailure(channel.clone()));
        match last_error.unwrap().downcast_ref::<GeneralFailure>() {
            Some(e) => Ok(Err(DataMessage::BackendRefused(channel.clone(),e.message().to_string()))),
            None => Err(DataMessage::CodeInvariantFailed("unexpected downcast error in backoff".to_string()))
        }
    }
}