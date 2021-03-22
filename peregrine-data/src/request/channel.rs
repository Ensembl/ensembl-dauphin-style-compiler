use anyhow::bail;
use lazy_static::lazy_static;
use std::future::Future;
use std::pin::Pin;
use std::fmt::{ self, Display, Formatter };
use anyhow::{ self, Context, anyhow as err };
use std::sync::Arc;
use regex::Regex;
use url::Url;
use serde_cbor::Value as CborValue;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_string };
use crate::util::message::DataMessage;

lazy_static! {
    static ref CHANNEL_RE: Regex = Regex::new(r"(.*?)\((.*)\)").unwrap();
}

pub trait ChannelIntegration {
    fn set_timeout(&self, channel: &Channel, timeout: f64);
    fn get_sender(&self,channel: Channel, prio: PacketPriority, data: CborValue) -> Pin<Box<dyn Future<Output=Result<CborValue,DataMessage>>>>;
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub enum ChannelLocation {
    HttpChannel(Url)
}

impl ChannelLocation {
    pub fn parse(base: &ChannelLocation, value: &str) -> anyhow::Result<ChannelLocation> {
        if let Some(parsed) = CHANNEL_RE.captures_iter(value).next() {
            match parsed.get(1).map(|x| x.as_str()) {
                Some("url") => Ok(ChannelLocation::HttpChannel(Url::parse(parsed.get(1).ok_or(err!("unparsable channel URL"))?.as_str())?)),
                Some("self") => Ok(base.clone()),
                _ => bail!("unparsable channel string!")
            }
        } else {
            bail!("unparsable channel string!");
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
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
            ChannelLocation::HttpChannel(url) => format!("url({})",url)
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

#[derive(Clone,PartialEq,Eq,Hash)]
pub enum PacketPriority {
    RealTime,
    Batch
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
    pub fn serialize(&self) -> Result<CborValue,DataMessage> {
        Ok(match self.0.as_ref() {
            ChannelLocation::HttpChannel(url) => CborValue::Array(vec![CborValue::Integer(0),CborValue::Text(url.to_string())])
        })
    }

    pub fn deserialize(value: &CborValue) -> anyhow::Result<Channel> {
        let values = cbor_array(value,2,false)?;
        let data = match cbor_int(&values[0],Some(0))? {
            0 => ChannelLocation::HttpChannel(Url::parse(&cbor_string(&values[1])?).map_err(|e| err!(e.to_string())).context("parsing URL")?),
            _ => Err(err!("bad channel type in deserialize"))?
        };
        Ok(Channel(Arc::new(data)))
    }
}
