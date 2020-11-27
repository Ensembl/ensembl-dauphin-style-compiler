use anyhow::{ anyhow as err };
use peregrine_core::{ Channel, ChannelLocation, PacketPriority, ChannelIntegration, lock };
use serde_cbor::Value as CborValue;
use crate::util::ajax::PgAjax;
use super::pgconsole::{ PgConsoleWeb };
use url::Url;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };

#[derive(Clone)]
pub struct PgChannel(PgConsoleWeb,Arc<Mutex<HashMap<Channel,Option<f64>>>>);

impl PgChannel {
    pub fn new(console: PgConsoleWeb) -> PgChannel {
        PgChannel(console,Arc::new(Mutex::new(HashMap::new())))
    }
}

fn add_priority(a: &Url, prio: PacketPriority) -> anyhow::Result<Url> {
    let mut z = a.clone();
    let mut path = z.path_segments_mut().map_err(|_| err!("cannot manipulate URL"))?;
    path.push(match prio {
        PacketPriority::RealTime => "hi",
        PacketPriority::Batch => "lo"
    });
    drop(path);
    Ok(z)
}

async fn send(channel: Channel, prio: PacketPriority, data: CborValue, timeout: Option<f64>) -> anyhow::Result<CborValue> {
    match channel.location().as_ref() {
        ChannelLocation::HttpChannel(url) => {
            let mut ajax = PgAjax::new("POST",&add_priority(url,prio)?);
            if let Some(timeout) = timeout {
                ajax.set_timeout(timeout);
            }
            ajax.set_body_cbor(&data)?;
            let out = ajax.get_cbor().await;
            out
        }
    }
}

/* using async_trait gives odd errors re Send */
impl ChannelIntegration for PgChannel {
    fn get_sender(&self,channel: Channel, prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>> {
        let timeout = lock!(self.1).get(&channel).and_then(|x| x.clone());
        Box::pin(send(channel,prio,data,timeout))
    }

    fn warn(&self, _channel: &Channel, msg: &str) {
        self.0.warn(msg);
    }

    fn error(&self, _channel: &Channel, msg: &str) {
        self.0.error(msg);
    }

    fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.1.lock().unwrap().insert(channel.clone(),Some(timeout));
    }
}
