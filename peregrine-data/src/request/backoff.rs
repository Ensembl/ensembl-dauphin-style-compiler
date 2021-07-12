use anyhow::bail;
use blackbox::blackbox_count;
use commander::cdr_timer;
use std::any::Any;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::failure::GeneralFailure;
use super::request::{ RequestType };
use crate::util::message::DataMessage;

const BACKOFF: &'static [u32] = &[ 0, 1, 1, 1, 100, 100, 100, 500, 500, 500, 5000, 5000, 5000 ];

pub struct Backoff(usize);

impl Backoff {
    pub fn new() -> Backoff { Backoff(0) }

    pub async fn wait(&mut self) -> anyhow::Result<()> {
        if self.0 >= BACKOFF.len() { bail!("too many retries"); }
        cdr_timer(BACKOFF[self.0] as f64).await;
        self.0 += 1;
        Ok(())
    }

    pub async fn backoff<S,R,F>(&mut self, manager: &mut RequestManager, req: R, channel: &Channel, prio: PacketPriority, verify: F)
                    -> Result<Result<Box<S>,DataMessage>,DataMessage>
                    where R: RequestType+Clone + 'static, S: 'static, F: Fn(&S) -> Option<GeneralFailure> {
        let channel = channel.clone();
        let mut last_error = None;
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let req2 = Box::new(req.clone());
            let resp = manager.execute(channel.clone(),prio.clone(),req2).await?;
            match resp.into_any().downcast::<S>() {
                Ok(s) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"success",1.);
                    match verify(&s) {
                        Some(_) => { last_error = Some(s as Box<dyn Any>); },
                        None => {
                            return Ok(Ok(s))
                        }
                    }
                },
                Err(resp) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1.);
                    match resp.downcast::<GeneralFailure>() {
                        Ok(e) => { 
                            manager.message(DataMessage::BackendRefused(channel.clone(),e.message().to_string()));
                            last_error = Some(e);
                        },
                        Err(e) => {
                            return Err(DataMessage::PacketError(channel,format!("unexpected response to request: {:?}",e)));
                        }
                    }
                }
            }
            manager.message(DataMessage::TemporaryBackendFailure(channel.clone()));

        }
        manager.message(DataMessage::FatalBackendFailure(channel.clone()));
        match last_error.unwrap().downcast_ref::<GeneralFailure>() {
            Some(e) => Ok(Err(DataMessage::BackendRefused(channel.clone(),e.message().to_string()))),
            None => Err(DataMessage::CodeInvariantFailed("unexpected downcast error in backoff".to_string()))
        }
    }
}