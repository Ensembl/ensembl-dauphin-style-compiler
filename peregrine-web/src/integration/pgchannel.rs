use anyhow::{ anyhow as err };
use peregrine_core::{ Channel, ChannelLocation, PacketPriority, ChannelIntegration };
use serde_cbor::Value as CborValue;
use crate::util::ajax::PgAjax;
use url::Url;
use std::future::Future;
use std::pin::Pin;

pub struct PgChannel();

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

async fn send(channel: Channel, prio: PacketPriority, data: CborValue) -> anyhow::Result<CborValue> {
    match channel.location().as_ref() {
        ChannelLocation::HttpChannel(url) => {
            let mut ajax = PgAjax::new("POST",&add_priority(url,prio)?);
            ajax.set_body_cbor(&data)?;
            ajax.get_cbor().await
        }
    }
}

/* using sync trait gives odd errors re Send */
impl ChannelIntegration for PgChannel {
    fn get_sender(&self,channel: Channel, prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=anyhow::Result<CborValue>>>> {
        Box::pin(send(channel,prio,data))
    }
}