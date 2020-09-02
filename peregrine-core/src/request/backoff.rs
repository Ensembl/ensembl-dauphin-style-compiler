use anyhow::{ bail, anyhow as err };
use blackbox::blackbox_count;
use commander::cdr_timer;
use owning_ref::RcRef;
use super::channel::{ Channel, PacketPriority };
use super::manager::RequestManager;
use super::failure::GeneralFailure;
use super::request::{ RequestType, ResponseType };

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
                    -> anyhow::Result<anyhow::Result<RcRef<dyn ResponseType,S>>>
                    where R: RequestType+Clone + 'static, S: 'static, F: Fn(&S) -> Option<GeneralFailure> {
        let channel = channel.clone();
        let mut last_error = None;
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let req2 = Box::new(req.clone());
            let resp = manager.execute(channel.clone(),prio.clone(),req2).await?;
            match resp.as_any().downcast_ref::<S>() {
                Some(b) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"success",1.);
                    match verify(&b) {
                        Some(_) => { last_error = Some(resp.clone()); },
                        None => {
                            let out = RcRef::new(resp.clone()).map(|x| {
                                x.as_any().downcast_ref::<S>().unwrap()
                            });
                            return Ok(Ok(out))                
                        }
                    }
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1.);    
                },
                None => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1.);
                    match resp.as_any().downcast_ref::<GeneralFailure>() {
                        Some(e) => { 
                            manager.error(&channel,e.message());
                            last_error = Some(resp.clone());
                        },
                        None => {
                            bail!("Unexpected response to request");
                        }
                    }
                }
            }
            manager.warn(&channel,&format!("temporary(?) failure of {}",channel.to_string()));

        }
        match last_error.unwrap().as_any().downcast_ref::<GeneralFailure>() {
            Some(e) => Ok(Err(err!(e.message().to_string()))),
            None => bail!("unexpected downcast error")
        }
    }

    /*
    pub async fn backoff2<S,R,F>(&mut self, manager: &mut RequestManager, req: R, channel: &Channel, prio: PacketPriority, verify: F)
                    -> anyhow::Result<anyhow::Result<S>>
                    where R: RequestType+Clone + 'static, S: 'static, F: Fn(&S) -> Option<GeneralFailure> {
        let channel = channel.clone();
        let mut last_error = None;
        while self.wait().await.is_ok() {
            let channel = channel.clone();
            let req2 = Box::new(req.clone());
            let resp = manager.execute(channel.clone(),prio.clone(),req2).await?;
            match resp.as_any().downcast_ref::<S>() {
                Some(b) => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"success",1.);
                    match verify(&b) {
                        Some(_) => { last_error = Some(resp.clone()); },
                        None => {
                            let out = RcRef::new(resp.clone()).map(|x| {
                                x.as_any().downcast_ref::<S>().unwrap()
                            });
                            return Ok(Ok(out))                
                        }
                    }
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1.);    
                },
                None => {
                    blackbox_count!(&format!("channel-{}",channel.to_string()),"failure",1.);
                    match resp.as_any().downcast_ref::<GeneralFailure>() {
                        Some(e) => { 
                            manager.error(&channel,e.message());
                            last_error = Some(resp.clone());
                        },
                        None => {
                            bail!("Unexpected response to request");
                        }
                    }
                }
            }
            manager.warn(&channel,&format!("temporary(?) failure of {}",channel.to_string()));

        }
        match last_error.unwrap().as_any().downcast_ref::<GeneralFailure>() {
            Some(e) => Ok(Err(err!(e.message().to_string()))),
            None => bail!("unexpected downcast error")
        }
    }
    */
}