use peregrine_toolkit::cbor::{cbor_into_drained_map, cbor_into_vec};
use std::collections::BTreeMap;
use std::mem::replace;
use std::sync::Arc;
use crate::core::channel::Channel;
use crate::core::programbundle::SuppliedBundle;
use super::request::CommandRequest;
use super::response::NewCommandResponse;
use serde_cbor::Value as CborValue;

pub struct RequestPacketBuilder {
    channel: Channel,
    requests: Vec<CommandRequest>
}

impl RequestPacketBuilder {
    pub fn new(channel: &Channel) -> RequestPacketBuilder {
        RequestPacketBuilder {
            channel: channel.clone(),
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: CommandRequest) {
        self.requests.push(request);
    }
}

#[derive(Clone)]
pub struct RequestPacket {
    channel: Channel,
    requests: Arc<Vec<CommandRequest>>
}

impl RequestPacket {
    pub fn new(builder: RequestPacketBuilder) -> RequestPacket {
        RequestPacket {
            channel: builder.channel.clone(),
            requests: Arc::new(builder.requests.clone())
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
        CborValue::Map(map)
    }
}

pub struct ResponsePacket {
    responses: Vec<NewCommandResponse>,
    programs: Vec<SuppliedBundle>
}

impl ResponsePacket {
    fn new() -> ResponsePacket {
        ResponsePacket {
            responses: vec![],
            programs: vec![]
        }
    }

    fn decode_responses(value: CborValue) -> Result<Vec<NewCommandResponse>,String> {
        Ok(cbor_into_vec(value)?.drain(..)
            .map(|v| NewCommandResponse::decode(v))
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
        for (k,v) in cbor_into_drained_map(value)?.drain(..) {
            match k.as_str() {
                "responses" => { responses = ResponsePacket::decode_responses(v)?; },
                "programs" => { programs = ResponsePacket::decode_programs(v)?; },
                _ => {}
            }
        }
        Ok(ResponsePacket { responses, programs })
    }

    fn add_response(&mut self, response: NewCommandResponse) {
        self.responses.push(response);
    }

    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<NewCommandResponse> {
        replace(&mut self.responses,vec![])
    }
}