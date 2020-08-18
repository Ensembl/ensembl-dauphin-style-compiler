use anyhow::{ bail };
use commander::cdr_timer;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::request::RequestType;

const BACKOFF: &'static [u32] = &[ 0, 1, 1, 1, 100, 100, 100, 500, 500, 500, 5000, 5000, 5000, 15000, 15000 ];

pub struct Backoff(usize);

impl Backoff {
    pub fn new() -> Backoff { Backoff(0) }

    pub async fn wait(&mut self) -> anyhow::Result<()> {
        if self.0 >= BACKOFF.len() { bail!("too many retries"); }
        cdr_timer(BACKOFF[self.0] as f64).await;
        self.0 += 1;
        Ok(())
    }

    pub async fn backoff_two_messages<S,F,R>(&mut self, manager: &mut RequestManager, req: R, channel: &Channel, prio: PacketPriority) -> anyhow::Result<Result<Box<S>,Box<F>>> where R: RequestType+Clone + 'static, S: 'static, F: 'static {
        let channel = channel.clone();
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let resp = manager.execute(channel.clone(),prio.clone(),Box::new(req.clone())).await?;
            match resp.into_any().downcast::<S>() {
                Ok(b) => {
                    return Ok(Ok(b));
                },
                Err(e) => {
                    match e.downcast::<F>() {
                        Ok(_) => {},
                        Err(_) => {
                            bail!("Unexpected response to request");
                        }
                    }
                }
            }
        }
        let resp = manager.execute(channel.clone(),prio,Box::new(req.clone())).await?;
        match resp.into_any().downcast::<F>() {
            Ok(e) => Ok(Err(e)),
            Err(_) => {
                bail!("Unexpected response to bootstrap");
            }
        }
    }

    pub async fn backoff_one_message<S,R,F>(&mut self, manager: &mut RequestManager, req: R, channel: &Channel, prio: PacketPriority, pred: F) -> anyhow::Result<Result<Box<S>,Box<S>>>
            where R: RequestType+Clone + 'static, S: 'static, F: Fn(&Box<S>) -> bool {
        let channel = channel.clone();
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let resp = manager.execute(channel.clone(),prio.clone(),Box::new(req.clone())).await?;
            match resp.into_any().downcast::<S>() {
                Ok(b) => {
                    if pred(&b) {
                        return Ok(Ok(b));
                    }
                },
                Err(_) => {
                    bail!("Unexpected response to request");
                }
            }
        }
        let resp = manager.execute(channel.clone(),prio,Box::new(req.clone())).await?;
        match resp.into_any().downcast::<S>() {
            Ok(b) => {
                if pred(&b) {
                    return Ok(Ok(b));
                } else {
                    return Ok(Err(b));
                }
            },
            Err(_) => {
                bail!("Unexpected response to request");
            }
        }
}
}