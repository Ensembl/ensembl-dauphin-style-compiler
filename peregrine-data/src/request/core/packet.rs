use futures::Future;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::serdetools::{st_field};
use serde::de::{Visitor, MapAccess, DeserializeSeed};
use serde::{Serialize, Deserializer};
use serde::ser::SerializeMap;
use std::any::Any;
use std::fmt;
use std::mem::replace;
use std::pin::Pin;
use std::sync::Arc;
use crate::core::channel::channelintegration::{ChannelMessageDecoder};
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::{PacketPriority, DataMessage, ChannelSender, BackendNamespace};
use crate::core::programbundle::SuppliedBundle;
use crate::core::version::VersionMetadata;
use super::request::MiniRequestAttempt;
use super::response::{MiniResponseAttempt, MiniResponseAttemptVecDeserialize};

#[allow(unused)] // used in debug_big_requests
const TOO_LARGE : usize = 100*1024;

#[cfg(debug_big_requests)]
use peregrine_toolkit::{warn};

#[derive(Clone)]
pub struct RequestPacketFactory {
    channel: BackendNamespace,
    priority: PacketPriority,
    metadata: VersionMetadata,
}

impl RequestPacketFactory {
    pub fn new(channel: &BackendNamespace, priority: &PacketPriority, metadata: &VersionMetadata) -> RequestPacketFactory {
        RequestPacketFactory {
            channel: channel.clone(),
            priority: priority.clone(),
            metadata: metadata.clone()
        }
    }

    pub fn create(&self) -> RequestPacketBuilder {
        RequestPacketBuilder::new(&self)
    }
}

pub struct RequestPacketBuilder {
    factory: RequestPacketFactory,
    requests: Vec<MiniRequestAttempt>
}

impl RequestPacketBuilder {
    fn new(factory: &RequestPacketFactory) -> RequestPacketBuilder {
        RequestPacketBuilder {
            factory: factory.clone(),
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: MiniRequestAttempt) {
        self.requests.push(request);
    }
}

#[derive(Clone)]
pub struct MaxiRequest {
    factory: RequestPacketFactory,
    requests: Arc<Vec<MiniRequestAttempt>>,
}

impl MaxiRequest {
    pub fn new(builder: RequestPacketBuilder) -> MaxiRequest {
        MaxiRequest {
            factory: builder.factory.clone(),
            requests: Arc::new(builder.requests.clone()),
        }
    }

    pub fn fail(&self) -> MaxiResponse {
        let mut response = MaxiResponse::empty(&self.factory.channel);
        for r in self.requests.iter() {
            response.add_response(r.fail());
        }
        response
    }

    pub fn requests(&self) -> &[MiniRequestAttempt] { &self.requests }
    pub fn channel(&self) -> &BackendNamespace { &self.factory.channel }
    pub fn metadata(&self) -> &VersionMetadata { &self.factory.metadata }

    pub(crate) fn sender(&self, sender: &WrappedChannelSender) -> Result<Pin<Box<dyn Future<Output=Result<MaxiResponse,Error>>>>,DataMessage> {
        let decoder = ChannelMessageDecoder::new(sender);
        Ok(sender.get_sender(&self.factory.priority,self.clone(),decoder))
    }
}

impl Serialize for MaxiRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: serde::Serializer {
        let mut map = serializer.serialize_map(Some(3))?;
        map.serialize_entry("channel",self.channel())?;
        map.serialize_entry("requests",self.requests())?;
        map.serialize_entry("version",self.metadata())?;
        map.end()
    }
}

pub struct MaxiResponse {
    channel: BackendNamespace,
    responses: Vec<MiniResponseAttempt>,
    programs: Vec<SuppliedBundle>
}

impl MaxiResponse {
    fn empty(channel: &BackendNamespace) -> MaxiResponse {
        MaxiResponse {
            channel: channel.clone(),
            responses: vec![],
            programs: vec![]
        }
    }

    fn add_response(&mut self, response: MiniResponseAttempt) {
        self.responses.push(response);
    }

    #[cfg(debug_big_requests)]
    fn check_big_requests(&self) {
        let total_size : usize = self.responses.iter().map(|x| x.total_size()).sum();
        if total_size > TOO_LARGE {
            warn!("excessively large maxi-response {} ({} elements)",total_size,self.responses.len());
        }
        for mini in &self.responses {
            if mini.total_size() > TOO_LARGE/5 {
                warn!("excessively large mini-response {}",mini.description());
                for (key,size) in mini.component_size().iter() {
                    if mini.total_size() > TOO_LARGE/15 {
                        warn!("excessively large mini-response internal key {} ({})",key,size);
                    }
                }
            }
        }
    }

    #[cfg(not(debug_big_requests))]
    fn check_big_requests(&self) {}

    pub(crate) fn channel(&self) -> &BackendNamespace { &self.channel }
    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<MiniResponseAttempt> {
        self.check_big_requests();
        replace(&mut self.responses,vec![])
    }
}

struct MaxiResponseVisitor(WrappedChannelSender,Arc<dyn Any>);

impl<'de> Visitor<'de> for MaxiResponseVisitor {
    type Value = MaxiResponse;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a MaxiResponse")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where M: MapAccess<'de> {
        let mut responses : Option<Vec<MiniResponseAttempt>> = None;
        let mut programs = None;
        let mut channel = None;
        while let Some(key) = access.next_key()? {
            match key {
                "responses" => { 
                    //total_size = Self::total_size(&v).ok().unwrap_or(0);
                    responses = Some(access.next_value_seed(MiniResponseAttemptVecDeserialize(self.0.clone(),self.1.clone()))?);
                },
                "programs" => { programs = access.next_value()? },
                "channel" => { channel = access.next_value()? },
                _ => {}
            }
        }
        let responses = st_field("responses",responses)?;
        let channel = st_field("channel",channel)?;
        let programs = st_field("programs",programs)?;
        Ok(MaxiResponse {
            channel, 
            responses, 
            programs,
        })
    }
}

#[derive(Clone)]
pub struct DeserializeData;

pub struct MaxiResponseDeserialize(pub(crate) WrappedChannelSender,pub(crate) Arc<dyn Any>);

impl<'de> DeserializeSeed<'de> for MaxiResponseDeserialize {
    type Value = MaxiResponse;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: Deserializer<'de> {
        deserializer.deserialize_seq(MaxiResponseVisitor(self.0.clone(),self.1.clone()))
    }
}
