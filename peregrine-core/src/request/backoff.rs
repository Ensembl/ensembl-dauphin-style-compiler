use anyhow::{ bail, anyhow as err };
use blackbox::blackbox_count;
use commander::cdr_timer;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::failure::GeneralFailure;
use super::request::RequestType;

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

    pub async fn backoff<S,R,F>(&mut self, manager: &mut RequestManager, req: R, channel: &Channel, prio: PacketPriority, verify: F) -> anyhow::Result<anyhow::Result<Box<S>>> 
                    where R: RequestType+Clone + 'static, S: 'static, F: Fn(&S) -> Option<GeneralFailure> {
        let channel = channel.clone();
        let mut last_error = None;
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let resp = manager.execute(channel.clone(),prio.clone(),Box::new(req.clone())).await?;
            match resp.into_any().downcast::<S>() {
                Ok(b) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"success",1);
                    match verify(&b) {
                        Some(e) => { last_error = Some(Box::new(e)); },
                        None => { return Ok(Ok(b)); }
                    }
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1);    
                },
                Err(e) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1);
                    match e.downcast::<GeneralFailure>() {
                        Ok(e) => { last_error = Some(e); },
                        Err(_) => {
                            bail!("Unexpected response to request");
                        }
                    }
                }
            }
            manager.warn(&channel,&format!("temporary(?) failure of {}",channel.to_string()));
        }
        Ok(Err(err!(last_error.unwrap().message().to_string())))
    }
}