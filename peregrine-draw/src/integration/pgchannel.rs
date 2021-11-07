use js_sys::Date;
use peregrine_data::{Channel, ChannelIntegration, ChannelLocation, PacketPriority, RequestPacket, ResponsePacket};
use serde_cbor::Value as CborValue;
use crate::util::ajax::PgAjax;
use peregrine_toolkit::url::Url;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{ Arc, Mutex };
use crate::util::message::Message;
use peregrine_data::DataMessage;
use peregrine_toolkit::lock;

#[derive(Clone)]
pub struct PgChannel(Arc<Mutex<HashMap<Channel,Option<f64>>>>,String);

impl PgChannel {
    pub fn new() -> PgChannel {
        let cache_buster = Date::now() as u64;
        PgChannel(Arc::new(Mutex::new(HashMap::new())),format!("{:016x}",cache_buster))
    }
}

fn add_priority(a: &Url, prio: PacketPriority, cache_buster: &str) -> Result<Url,Message> {
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
            let mut ajax = PgAjax::new("POST",&add_priority(url,prio,cache_buster)?);
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

pub(crate) fn ser_wrap<T,E>(value: Result<T,E>) -> Result<T,DataMessage> where E: ToString {
    value.map_err(|e| DataMessage::CodeInvariantFailed(format!("cannot serialzie: {}",e.to_string())))
}

async fn send_wrap(channel: Channel, prio: PacketPriority, packet: RequestPacket, timeout: Option<f64>, cache_buster: String) -> Result<ResponsePacket,DataMessage> {
    let xxx_value = ser_wrap(serde_cbor::to_vec(&packet))?;
    let data = ser_wrap(serde_cbor::from_slice(&xxx_value))?;
    let data = send(channel,prio,data,timeout,&cache_buster).await.map_err(|e| DataMessage::TunnelError(Arc::new(Mutex::new(e))))?;
    let xxx_data = ser_wrap(serde_cbor::to_vec(&data))?;
    let response = ser_wrap(serde_cbor::from_slice::<ResponsePacket>(&xxx_data))?;
    Ok(response)
}

/* using async_trait gives odd errors re Send */
impl ChannelIntegration for PgChannel {
    fn get_sender(&self,channel: Channel, prio: PacketPriority, packet: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>> {
        let timeout = lock!(self.0).get(&channel).and_then(|x| x.clone());
        Box::pin(send_wrap(channel,prio,packet,timeout,self.1.clone()))
    }

    fn set_timeout(&self, channel: &Channel, timeout: f64) {
        self.0.lock().unwrap().insert(channel.clone(),Some(timeout));
    }
}
