use anyhow::{ self };
use serde_derive::Serialize;
use std::collections::{ HashMap };
use std::mem::replace;
use std::rc::Rc;
use std::sync::Arc;
use serde_cbor::Value as CborValue;
use super::channel::Channel;
use super::programbundle::SuppliedBundle;
use super::request::{CommandRequest, NewCommandResponse, ResponseBuilderType};
use crate::{ChannelLocation, DataMessage};
use crate::util::cbor::{ cbor_array, cbor_map };

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
    pub builders: Rc<HashMap<u8,Box<dyn ResponseBuilderType>>>
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
    pub fn new_packet(&self, value: &CborValue) -> Result<ResponsePacket,DataMessage> {
        ResponsePacket::deserialize(value)
    }
}

fn packet_wrap<T, E: ToString>(v: Result<T,E>) -> Result<T,DataMessage> {
    v.map_err(|e| DataMessage::PacketError(Channel::new(&ChannelLocation::None),e.to_string()))
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

    fn add_response(&mut self, response: NewCommandResponse) {
        self.responses.push(response);
    }

    pub(crate) fn programs(&self) -> &[SuppliedBundle] { &self.programs }
    pub(crate) fn take_responses(&mut self) -> Vec<NewCommandResponse> {
        replace(&mut self.responses,vec![])
    }

    fn deserialize(value: &CborValue,) -> Result<ResponsePacket,DataMessage> {
        let values = packet_wrap(cbor_map(value,&["responses","programs"]))?;
        let mut responses = vec![];
        for v in packet_wrap(cbor_array(&values[0],0,true))? {
            let xxx_data = packet_wrap(serde_cbor::to_vec(v))?;
            let v = packet_wrap(serde_cbor::from_slice::<NewCommandResponse>(&xxx_data))?;
            responses.push(v);
        }
        let programs : anyhow::Result<_> = packet_wrap(cbor_array(&values[1],0,true))?.iter().map(|x| SuppliedBundle::new(x)).collect();
        Ok(ResponsePacket {
            responses,
            programs: packet_wrap(programs)?
        })
    }
}
