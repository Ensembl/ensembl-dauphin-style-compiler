use futures::Future;
use peregrine_toolkit::cbor::{cbor_into_drained_map, cbor_into_vec};
use peregrine_toolkit::error::Error;
use serde::Serialize;
use serde::ser::SerializeMap;
use std::mem::replace;
use std::pin::Pin;
use std::sync::Arc;
use crate::core::channel::wrappedchannelsender::WrappedChannelSender;
use crate::{PacketPriority, DataMessage, ChannelSender, BackendNamespace};
use crate::core::programbundle::SuppliedBundle;
use crate::core::version::VersionMetadata;
use super::request::MiniRequestAttempt;
use super::response::{BackendResponseAttempt};
use serde_cbor::Value as CborValue;

#[cfg(debug_big_requests)]
const TOO_LARGE : usize = 100*1024;

#[cfg(debug_big_requests)]
use peregrine_toolkit::{warn , cbor::cbor_as_vec };

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

    pub fn fail(&self) -> ResponsePacket {
        let mut response = ResponsePacket::empty(&self.factory.channel);
        for r in self.requests.iter() {
            response.add_response(r.fail());
        }
        response
    }

    pub fn requests(&self) -> &[MiniRequestAttempt] { &self.requests }
    pub fn channel(&self) -> &BackendNamespace { &self.factory.channel }
    pub fn metadata(&self) -> &VersionMetadata { &self.factory.metadata }

    pub(crate) fn sender(&self, sender: &WrappedChannelSender) -> Result<Pin<Box<dyn Future<Output=Result<ResponsePacket,Error>>>>,DataMessage> {
        Ok(sender.get_sender(&self.factory.priority,self.clone()))
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

pub struct ResponsePacket {
    channel: BackendNamespace,
    responses: Vec<BackendResponseAttempt>,
    programs: Vec<SuppliedBundle>,
    #[cfg(debug_big_requests)]
    total_size: usize
}

impl ResponsePacket {
    fn empty(channel: &BackendNamespace) -> ResponsePacket {
        ResponsePacket {
            channel: channel.clone(),
            responses: vec![],
            programs: vec![],
            #[cfg(debug_big_requests)]
            total_size: 0
        }
    }

    #[cfg(debug_big_requests)]
    fn total_size(value: &CborValue) -> Result<usize,String> {
        Ok(cbor_as_vec(value)?.iter()
            .map(|v| BackendResponseAttempt::total_size(v))
            .collect::<Result<Vec<usize>,String>>()?.iter().sum())
    }

    #[cfg(not(debug_big_requests))]
    fn total_size(_value: &CborValue) -> Result<usize,String> {
        Ok(0)
    }

    fn decode_responses(value: CborValue) -> Result<Vec<BackendResponseAttempt>,String> {
        Ok(cbor_into_vec(value)?.drain(..)
            .map(|v| BackendResponseAttempt::decode(v))
            .collect::<Result<_,_>>()?)
    }

    fn decode_programs(value: CborValue) -> Result<Vec<SuppliedBundle>,String> {
        Ok(cbor_into_vec(value)?.drain(..)
            .map(|v| SuppliedBundle::decode(v))
            .collect::<Result<_,_>>()?)
    }

    pub fn decode(value: CborValue) -> Result<ResponsePacket,String> {
        let mut responses = vec![];
        let mut programs= vec![];
        let mut channel = None;
        #[allow(unused)] let mut total_size = 0;
        for (k,v) in cbor_into_drained_map(value)?.drain(..) {
            match k.as_str() {
                "responses" => {
                    total_size = Self::total_size(&v).ok().unwrap_or(0);
                    responses = ResponsePacket::decode_responses(v)?;
                },
                "programs" => { programs = ResponsePacket::decode_programs(v)?; },
                "channel" => { channel = Some(v); }
                _ => {}
            }
        }
        let channel = if let Some(channel) = channel { channel } else {
            return Err("missing channel in response".to_string());
        };
        Ok(ResponsePacket {
            channel: BackendNamespace::decode(channel)?,
            responses, programs, 
            #[cfg(debug_big_requests)]
            total_size
        })
    }

    fn add_response(&mut self, response: BackendResponseAttempt) {
        self.responses.push(response);
    }

    #[cfg(debug_big_requests)]
    fn check_big_requests(&self) {
        if self.total_size > TOO_LARGE {
            warn!("excessively large response size {} ({} elements)",self.total_size,self.responses.len());
        }
    }

    #[cfg(not(debug_big_requests))]
    fn check_big_requests(&self) {}

    pub(crate) fn channel(&self) -> &BackendNamespace { &self.channel }
    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<BackendResponseAttempt> {
        self.check_big_requests();
        replace(&mut self.responses,vec![])
    }
}
