use serde_derive::{Deserialize, Serialize};
use std::mem::replace;
use std::sync::Arc;
use serde_cbor::Value as CborValue;
use super::channel::Channel;
use super::programbundle::SuppliedBundle;
use super::request::{CommandRequest, NewCommandResponse};
use crate::{ChannelLocation, DataMessage};

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

fn packet_wrap<T, E: ToString>(v: Result<T,E>) -> Result<T,DataMessage> {
    v.map_err(|e| DataMessage::PacketError(Channel::new(&ChannelLocation::None),e.to_string()))
}

#[derive(Deserialize)]
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

    pub(crate) fn process(value: &CborValue) -> Result<ResponsePacket,DataMessage> {
        let xxx_data = packet_wrap(serde_cbor::to_vec(value))?;
        let v = packet_wrap(serde_cbor::from_slice::<ResponsePacket>(&xxx_data))?;
        Ok(v)
    }
}
