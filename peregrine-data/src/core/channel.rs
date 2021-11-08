use anyhow::bail;
use serde::{Deserialize, Deserializer, Serializer};
use serde::de::{self, SeqAccess, Unexpected, Visitor};
use std::future::Future;
use std::pin::Pin;
use std::fmt::{ self, Display, Formatter };
use anyhow::{ self };
use std::sync::Arc;
use peregrine_toolkit::url::Url;
use serde_cbor::Value as CborValue;
use crate::{RequestPacket, ResponsePacket};
use crate::util::message::DataMessage;
use serde_derive::{ Serialize };
use peregrine_toolkit::serde::{de_seq_next, de_wrap};
use peregrine_toolkit::envaryseq;

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

impl Channel {
    pub fn deserialize(value: &CborValue) -> Result<Channel,serde_cbor::Error> {
        let xxx_bytes = serde_cbor::to_vec(value)?;
        serde_cbor::from_slice(&xxx_bytes)
    }
}

impl serde::Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self.0.as_ref() {
            ChannelLocation::HttpChannel(url) => envaryseq!(serializer,0,url.to_string()),
            ChannelLocation::None => envaryseq!(serializer,1)
        }
    }
}

struct ChannelVisitor;

impl<'de> Visitor<'de> for ChannelVisitor {
    type Value = Channel;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f,"a channel") }

    fn visit_seq<S>(self, mut seq: S) -> Result<Channel,S::Error> where S: SeqAccess<'de> {
        let data = match de_seq_next(&mut seq)? {
            0 => ChannelLocation::HttpChannel(de_wrap(Url::parse(de_seq_next(&mut seq)?))?),
            1 => ChannelLocation::None,
            _ => Err(de::Error::invalid_value(Unexpected::Str(&"out-of range integer"),&"in-range integer"))?
        };
        Ok(Channel(Arc::new(data)))
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Channel, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_seq(ChannelVisitor)
    }
}
