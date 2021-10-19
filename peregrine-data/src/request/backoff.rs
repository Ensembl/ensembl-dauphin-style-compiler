use commander::cdr_timer;
use std::any::Any;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::failure::GeneralFailure;
use super::request::{RequestType, ResponseType};
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

    fn downcast<S: 'static,F>(&self, resp: Box<dyn ResponseType>, verify: F) -> Result<Result<Box<S>,Box<dyn Any>>,DataMessage> where F: Fn(&S) -> bool {
        match resp.into_any().downcast::<S>() {
            Ok(s) => {
                /* Got expected response */
                if verify(&s) {
                    /* seemed to "work"! */
                    return Ok(Ok(s));
                } else {
                    /* but somehow failed */
                    return Ok(Err(s as Box<dyn Any>));
                }
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

    pub async fn backoff<S,R,F>(&mut self, req: R, verify: F) -> Result<Result<Box<S>,DataMessage>,DataMessage>
                    where R: RequestType+Clone + 'static, S: 'static, F: Fn(&S) -> bool {
        let channel = self.channel.clone();
        let mut last_error = None;
        for _ in 0..5 { // XXX configurable
            let resp = self.manager.execute(channel.clone(),self.priority.clone(),Box::new(req.clone())).await?;
            match self.downcast(resp,&verify)? {
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