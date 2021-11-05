use anyhow::{ self, Context, anyhow as err };
use serde::Serialize;
use std::collections::{ HashMap };
use std::mem::replace;
use std::rc::Rc;
use std::sync::Arc;
use serde_cbor::Value as CborValue;
use super::channel::Channel;
use super::programbundle::SuppliedBundle;
use super::request::{CommandRequest, CommandResponse, ResponseBuilderType};
use crate::util::cbor::{ cbor_array, cbor_int, cbor_map };

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

#[derive(Serialize,Clone)]
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
}

pub struct ResponsePacketBuilderBuilder {
    builders: HashMap<u8,Box<dyn ResponseBuilderType>>
}

pub struct ResponsePacketBuilder {
    builders: Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>
}

impl ResponsePacketBuilderBuilder {
    pub fn new() -> ResponsePacketBuilderBuilder {
        ResponsePacketBuilderBuilder {
            builders: HashMap::new()
        }
    }

    pub fn register(&mut self, type_index: u8, rbt: Box<dyn ResponseBuilderType>) {
        self.builders.insert(type_index,rbt);
    }

    pub fn build(self) -> ResponsePacketBuilder {
        ResponsePacketBuilder {
            builders: Rc::new(self.builders)
        }
    }
}

impl ResponsePacketBuilder {
    pub fn new_packet(&self, value: &CborValue) -> anyhow::Result<ResponsePacket> {
        ResponsePacket::deserialize(value,&self.builders).context("parsing server response")
    }
}

pub struct ResponsePacket {
    responses: Vec<CommandResponse>,
    programs: Vec<SuppliedBundle>
}

impl ResponsePacket {
    fn new() -> ResponsePacket {
        ResponsePacket {
            responses: vec![],
            programs: vec![]
        }
    }

    fn add_response(&mut self, response: CommandResponse) {
        self.responses.push(response);
    }

    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<CommandResponse> {
        replace(&mut self.responses,vec![])
    }

    fn deserialize_response(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<CommandResponse> {
        let values = cbor_array(value,2,false)?;
        let msgid = cbor_int(&values[0],None)? as u64;
        let response = cbor_array(&values[1],2,false)?;
        let builder = builders.get(&(cbor_int(&response[0],Some(255))? as u8)).ok_or(err!("bad response type"))?;
        let payload = builder.deserialize(&response[1]).with_context(
            || format!("deserializing individual response payload (type {})",cbor_int(&response[0],None).unwrap_or(-1))
        )?;
        Ok(CommandResponse::new(msgid,payload))
    }

    fn deserialize(value: &CborValue, builders: &Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>) -> anyhow::Result<ResponsePacket> {
        let values = cbor_map(value,&["responses","programs"])?;
        let mut responses = vec![];
        for v in cbor_array(&values[0],0,true)? {
            responses.push(ResponsePacket::deserialize_response(v,builders).with_context(
                || format!("deserializing individual response payload (type {})",cbor_int(&values[1],None).unwrap_or(-1))
            )?);
        }
        let programs : anyhow::Result<_> = cbor_array(&values[1],0,true)?.iter().map(|x| SuppliedBundle::new(x)).collect();
        Ok(ResponsePacket {
            responses,
            programs: programs?
        })
    }
}
