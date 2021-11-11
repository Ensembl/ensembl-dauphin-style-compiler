use anyhow::bail;
use peregrine_toolkit::cbor::{cbor_as_number, cbor_as_str, cbor_into_vec, check_array_len, check_array_min_len};
use std::future::Future;
use std::pin::Pin;
use std::fmt::{ self, Display, Formatter };
use anyhow::{ self };
use std::sync::Arc;
use peregrine_toolkit::url::Url;
use crate::{RequestPacket, ResponsePacket};
use crate::util::message::DataMessage;
use serde_derive::{ Serialize };
use serde_cbor::Value as CborValue;

fn parse_channel(value: &str) -> anyhow::Result<(String,String)> {
    if value.ends_with(")") {
        if let Some(first_idx) = value.find("(") {
            let mut first = value.to_string();
            let mut rest = first.split_off(first_idx)[1..].to_string();
            if rest.len() != 0 {
                rest = rest[0..(rest.len()-1)].to_string();
            }
            return Ok((first,rest))
        }
    }
    bail!("unparsable channel string!");
}

pub trait ChannelIntegration {
    fn set_supported_versions(&self, supports: Option<&[u32]>, version: u32);
    fn set_timeout(&self, channel: &Channel, timeout: f64);
    fn get_sender(&self,channel: Channel, prio: PacketPriority, data: RequestPacket) -> Pin<Box<dyn Future<Output=Result<ResponsePacket,DataMessage>>>>;
}

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub enum ChannelLocation {
    None,
    HttpChannel(Url)
}

impl ChannelLocation {
    pub fn parse(base: &ChannelLocation, value: &str) -> anyhow::Result<ChannelLocation> {
        let (first,rest) = parse_channel(value)?;
        match first.as_str() {
            "url" => Ok(ChannelLocation::HttpChannel(Url::parse(&rest)?)),
            "self" => Ok(base.clone()),
            "none" => Ok(ChannelLocation::None),
            _ => bail!("unparsable channel string!")
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct Channel(Arc<ChannelLocation>);

impl Channel {
    pub fn new(location: &ChannelLocation) -> Channel {
        Channel(Arc::new(location.clone()))
    }

    pub fn decode(value: CborValue) -> Result<Channel,String> {
        let value = cbor_into_vec(value)?;
        check_array_min_len(&value,1)?;
        let data = match cbor_as_number(&value[0])? {
            0 => {
                check_array_len(&value,2)?;
                ChannelLocation::HttpChannel(Url::parse(&cbor_as_str(&value[1])?).map_err(|e| e.to_string())?)
            },
            1 => ChannelLocation::None,
            x => { 
                return Err(format!("channel type out-of range: {}",x));
            }
        };
        Ok(Channel(Arc::new(data)))
    }

    pub fn parse(base: &Channel, value: &str) -> anyhow::Result<Channel> {
        Ok(Channel(Arc::new(ChannelLocation::parse(&base.0,value)?)))
    }

    pub(crate) fn channel_name(&self) -> String {
        match self.0.as_ref() {
            ChannelLocation::HttpChannel(url) => format!("url({})",url),
            ChannelLocation::None => format!("none()")
        }
    }

    pub fn location(&self) -> Arc<ChannelLocation> {
        self.0.clone()
    }

    pub fn encode(&self) -> CborValue {
        let v = match self.0.as_ref() {
            ChannelLocation::HttpChannel(url) => vec![CborValue::Integer(0),CborValue::Text(url.to_string())],
            ChannelLocation::None => vec![CborValue::Integer(1)]
        };
        CborValue::Array(v)
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f,"{}",self.channel_name())
    }
}

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,Serialize)]
pub enum PacketPriority {
    RealTime,
    Batch
}

impl PacketPriority {
    pub fn index(&self) -> usize {
        match self {
            PacketPriority::RealTime => 0,
            PacketPriority::Batch => 1
        }
    }
}

impl Display for PacketPriority {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PacketPriority::RealTime => write!(f,"real-time"),
            PacketPriority::Batch => write!(f,"batch")
        }
    }
}
