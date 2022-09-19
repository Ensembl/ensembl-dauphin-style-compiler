use js_sys::Date;
use peregrine_data::{Channel, ChannelIntegration, ChannelLocation, PacketPriority, RequestPacket, ResponsePacket, ChannelSender};
use serde_cbor::Value as CborValue;
use crate::util::ajax::PgAjax;
use peregrine_toolkit::url::Url;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use peregrine_data::DataMessage;
use peregrine_toolkit::{lock};

#[derive(Clone)]
pub struct NetworkChannel(Arc<Mutex<HashMap<Channel,Option<f64>>>>,String);

impl NetworkChannel {
    pub fn new() -> NetworkChannel {
        let cache_buster = Date::now() as u64;
        NetworkChannel(Arc::new(Mutex::new(HashMap::new())),format!("{:016x}",cache_buster))
    }
}

fn add_priority(a: &Url, prio: &PacketPriority, cache_buster: &str) -> Result<Url,Message> {
    let z = a.add_path_segment(match prio {
        PacketPriority::RealTime => "hi",
        PacketPriority::Batch => "lo"
    }).map_err(|_| Message::CodeInvariantFailed(format!("cannot manipulate URL")))?;
    let z = z.add_query_parameter(&format!("stamp={}",cache_buster))
        .map_err(|_| Message::CodeInvariantFailed(format!("cannot manipulate URL")))?;
    Ok(z)
}

async fn send(channel: Channel, prio: PacketPriority, data: CborValue, timeout: Option<f64>, cache_buster: &str) -> Result<CborValue,Message> {
    match channel.location().as_ref() {
        ChannelLocation::HttpChannel(url) => {
            let mut ajax = PgAjax::new("POST",&add_priority(url,&prio,cache_buster)?);
            if let Some(timeout) = timeout {
                ajax.set_timeout(timeout);
            }
            ajax.set_body_cbor(&data)?;
            let out = ajax.get_cbor().await;
            out
        },
        ChannelLocation::None => {
            return Err(Message::BadBackendConnection(format!("Cannot connect to the none() channel, by definition it deosn't exist")))
        }
    }
}

async fn send_wrap(channel: Channel, prio: PacketPriority, packet: RequestPacket, timeout: Option<f64>, cache_buster: String) -> Result<ResponsePacket,DataMessage> {
    let channel2 = channel.clone();
    let data = packet.encode();
    let data = send(channel,prio,data,timeout,&cache_buster).await.map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
    let response = ResponsePacket::decode(data).map_err(|e| DataMessage::PacketError(channel2,e))?;
    Ok(response)
}

pub struct NetworkChannelSender {
    channel: Channel,
    timeout: Option<f64>,
    cache_buster: String
}

impl ChannelSender for NetworkChannelSender {
    fn get_sender(&self, prio: &PacketPriority, packet: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>> {
        Box::pin(send_wrap(self.channel.clone(),prio.clone(),packet,self.timeout,self.cache_buster.clone()))
    }
}

/* using async_trait gives odd errors re Send */
impl ChannelIntegration for NetworkChannel {
    fn make_sender(&self, channel: &Channel) -> Option<Arc<dyn ChannelSender>> {
        Some(Arc::new(NetworkChannelSender {
            channel: channel.clone(),
            timeout: lock!(self.0).get(channel).cloned().flatten(),
            cache_buster: self.1.clone()
        }))
    }

    fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.0.lock().unwrap().insert(channel.clone(),Some(timeout));
    }
}
