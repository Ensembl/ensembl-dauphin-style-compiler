use peregrine_toolkit::cbor::{cbor_into_drained_map, cbor_into_vec};
use std::collections::BTreeMap;
use std::mem::replace;
use std::sync::Arc;
use crate::core::channel::Channel;
use crate::core::programbundle::SuppliedBundle;
use crate::core::version::VersionMetadata;
use super::request::BackendRequestAttempt;
use super::response::{BackendResponseAttempt};
use serde_cbor::Value as CborValue;

#[cfg(debug_big_requests)]
const TOO_LARGE : usize = 100*1024;

#[cfg(debug_big_requests)]
use peregrine_toolkit::{warn , cbor::cbor_as_vec };

pub struct RequestPacketBuilder {
    channel: Channel,
    requests: Vec<BackendRequestAttempt>
}

impl RequestPacketBuilder {
    pub fn new(channel: &Channel) -> RequestPacketBuilder {
        RequestPacketBuilder {
            channel: channel.clone(),
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: BackendRequestAttempt) {
        self.requests.push(request);
    }
}

#[derive(Clone)]
pub struct RequestPacket {
    channel: Channel,
    requests: Arc<Vec<BackendRequestAttempt>>,
    metadata: VersionMetadata
}

impl RequestPacket {
    pub fn new(builder: RequestPacketBuilder, metadata: &VersionMetadata) -> RequestPacket {
        RequestPacket {
            channel: builder.channel.clone(),
            requests: Arc::new(builder.requests.clone()),
            metadata: metadata.clone()
        }
    }

    pub fn fail(&self) -> ResponsePacket {
        let mut response = ResponsePacket::new();
        for r in self.requests.iter() {
            response.add_response(r.fail());
        }
        response
    }

    pub fn encode(&self) -> CborValue {
        let mut map = BTreeMap::new();
        map.insert(CborValue::Text("channel".to_string()), self.channel.encode());
        let requests = self.requests.iter().map(|r| r.encode()).collect::<Vec<_>>();
        map.insert(CborValue::Text("requests".to_string()),CborValue::Array(requests));
        map.insert(CborValue::Text("version".to_string()),self.metadata.encode());
        CborValue::Map(map)
    }
}

pub struct ResponsePacket {
    responses: Vec<BackendResponseAttempt>,
    programs: Vec<SuppliedBundle>,
    #[cfg(debug_big_requests)]
    total_size: usize
}

impl ResponsePacket {
    fn new() -> ResponsePacket {
        ResponsePacket {
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
        #[allow(unused)]
        let mut total_size = 0;
        for (k,v) in cbor_into_drained_map(value)?.drain(..) {
            match k.as_str() {
                "responses" => {
                    total_size = Self::total_size(&v).ok().unwrap_or(0);
                    responses = ResponsePacket::decode_responses(v)?;
                },
                "programs" => { programs = ResponsePacket::decode_programs(v)?; },
                _ => {}
            }
        }
        Ok(ResponsePacket { 
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

    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<BackendResponseAttempt> {
        self.check_big_requests();
        replace(&mut self.responses,vec![])
    }
}
