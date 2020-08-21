use anyhow::{ self, Context, anyhow as err };
use std::collections::{ BTreeMap, HashMap };
use std::mem::replace;
use std::rc::Rc;
use serde_cbor::Value as CborValue;
use super::channel::Channel;
use super::request::{ ResponseBuilderType, CommandResponse, CommandRequest };
use super::program::SuppliedBundle;
use crate::util::cbor::{ cbor_array, cbor_int, cbor_map };

pub struct RequestPacket {
    requests: Vec<CommandRequest>
}

impl RequestPacket {
    pub fn new() -> RequestPacket {
        RequestPacket {
            requests: vec![]
        }
    }

    pub fn add(&mut self, request: CommandRequest) {
        self.requests.push(request);
    }

    pub fn serialize(&self, channel: &Channel) -> anyhow::Result<CborValue> {
        let mut map = BTreeMap::new();
        let mut requests = vec![];
        for r in &self.requests {
            requests.push(r.serialize()?);
        }
        map.insert(CborValue::Text("requests".to_string()),CborValue::Array(requests));
        map.insert(CborValue::Text("channel".to_string()),channel.serialize()?);
        Ok(CborValue::Map(map))
    }

    pub fn fail(&self) -> ResponsePacket {
        let mut response = ResponsePacket::new();
        for r in &self.requests {
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
        let values = cbor_array(value,3,false)?;
        let msgid = cbor_int(&values[0],None)? as u64;
        let builder = builders.get(&(cbor_int(&values[1],Some(255))? as u8)).ok_or(err!("bad response type"))?;
        let payload = builder.deserialize(&values[2]).with_context(
            || format!("deserializing individual response payload (type {})",cbor_int(&values[1],None).unwrap_or(-1))
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
